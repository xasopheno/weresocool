use crate::voice::{SampleInfo, Voice};
use std::f64::consts::PI;
use weresocool_ast::OscType;
use weresocool_shared::r_to_f64;

const TAU: f64 = PI * 2.0;

// Per-drum loudness compensation. These were calibrated when gain.rs was
// applying a /3 attenuation to non-sine osc types; now that drums skip that
// attenuation, the constants are reduced ~3x to keep mix levels stable while
// the transient survives.
const KICK_GAIN: f64 = 0.4;
const SNARE_GAIN: f64 = 0.8;
const HIHAT_GAIN: f64 = 0.7;

// Default tuning when info.frequency falls outside the musical range we trust
// (e.g. silence ops setting freq to 0). Lets `Kick` still sound like a kick
// even when no header frequency is set.
const KICK_DEFAULT_FREQ: f64 = 60.0;
const SNARE_DEFAULT_FREQ: f64 = 185.0;
const HIHAT_DEFAULT_FREQ: f64 = 320.0;

/// Fast deterministic noise from sample index using a simple hash function.
/// Replaces per-sample RNG calls for better performance and reproducibility.
#[inline]
fn fast_noise(index: usize, seed: u64) -> f64 {
    let x = (index as u64).wrapping_mul(seed);
    let x = x ^ (x >> 33);
    let x = x.wrapping_mul(0xff51afd7ed558ccd);
    let x = x ^ (x >> 33);
    // Convert to range [-1, 1]
    ((x as i64) as f64) / (i64::MAX as f64)
}

/// Deterministic noise in range [-0.5, 0.5] for phase modulation
#[inline]
fn noise_offset(index: usize) -> f64 {
    fast_noise(index, 0x9E3779B97F4A7C15) * 0.5
}

/// Pink noise approximation using Voss-McCartney algorithm (very cheap).
/// Produces approximately -3 dB/octave rolloff.
#[inline]
fn pink_noise(index: usize, seed: u64) -> f64 {
    let w1 = fast_noise(index, seed);
    let w2 = fast_noise(index / 2, seed ^ 0x123456789);
    let w3 = fast_noise(index / 4, seed ^ 0x987654321);
    (w1 + w2 + w3) / 3.0
}

/// Soft clipper that adds odd harmonics - mimics membrane nonlinearity during decay.
/// drive > 1.0 increases saturation, drive = 1.0 is nearly linear.
#[inline]
fn soft_saturate(x: f64, drive: f64) -> f64 {
    let driven = x * drive;
    driven / (1.0 + driven.abs())
}

/// White noise with first-order differentiation: `y[i] = x[i] - x[i-1]`.
/// Frequency response is `|H(f)| = 2*sin(π*f/fs)` — about +18 dB/decade tilt.
/// This is what gives snare wires their "tssss" character instead of dull "shhh".
/// Stateless because `fast_noise` is a pure function of the index.
#[inline]
fn bright_noise(index: usize, seed: u64) -> f64 {
    let n0 = fast_noise(index, seed);
    let n1 = fast_noise(index.saturating_sub(1), seed);
    n0 - n1
}

/// Comb-style "band emphasis" noise: `y[i] = x[i] - x[i - tap]`. First spectral
/// peak sits at `fs / (2 * tap)` Hz. Cheap way to push noise into a register
/// without running an actual biquad — perfect for snare crack (~3-5 kHz) and
/// hihat air (~8-12 kHz).
#[inline]
fn comb_noise(index: usize, seed: u64, tap: usize) -> f64 {
    let n0 = fast_noise(index, seed);
    let n1 = fast_noise(index.saturating_sub(tap), seed);
    n0 - n1
}

/// Pseudo-bandpass noise: sum two comb stages with different taps to create
/// a broader peak with notches above and below. Used for the dense modal noise
/// in the hihat shimmer.
#[inline]
fn metal_noise(index: usize, seed: u64) -> f64 {
    let a = comb_noise(index, seed, 2);       // peak ~12 kHz @ 48k
    let b = comb_noise(index, seed ^ 0xA1B2C3D4, 4);  // peak ~6 kHz
    a * 0.6 + b * 0.4
}

/// Multi-stage transient envelope for realistic drum attacks.
/// Returns transient contribution that should be added to the main tone.
///
/// Stages:
/// 1. Spike (0 to spike_dur): Sharp attack spike
/// 2. Dip (spike_dur to spike_dur + dip_dur): Negative phase (membrane recoil)
/// 3. Settle (after): Returns to 0, main tone takes over
#[inline]
fn transient_envelope(t: f64, spike_curve: f64, dip_amount: f64) -> f64 {
    let spike_dur = 0.001;  // 1ms spike
    let dip_dur = 0.002;    // 2ms dip

    if t < spike_dur {
        // Stage 1: Sharp attack spike
        1.0 - (t / spike_dur).powf(spike_curve)
    } else if t < spike_dur + dip_dur {
        // Stage 2: Negative dip (membrane recoil)
        let dip_t = (t - spike_dur) / dip_dur;
        -dip_amount * (1.0 - dip_t).powi(2)
    } else {
        // Stage 3: Settle to 0
        0.0
    }
}

// ═══════════════════════════════════════════════════════════════════════
// BIQUAD FILTERS — RBJ Audio EQ Cookbook coefficients, TDF2 state
//
// Two state values per filter (`z1`, `z2`). Coefficients computed once at
// note start from cutoff + Q. Stable, cheap, and gives real resonant filter
// behaviour — the difference between "synth that sounds like a synth" and
// "synth that sounds like a drum."
// ═══════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Biquad {
    b0: f64, b1: f64, b2: f64,
    a1: f64, a2: f64,
    z1: f64, z2: f64,
}

impl Biquad {
    /// Resonant lowpass — natural for body/shell shaping.
    pub fn lowpass(&mut self, sr: f64, cutoff: f64, q: f64) {
        let cutoff = cutoff.clamp(20.0, sr * 0.45);
        let omega = TAU * cutoff / sr;
        let (so, co) = omega.sin_cos();
        let alpha = so / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - co) * 0.5) / a0;
        self.b1 = (1.0 - co) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * co) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Constant-skirt-gain resonant bandpass — peaks at cutoff with bandwidth ~ cutoff/Q.
    /// This is what turns "white noise" into "metallic noise."
    pub fn bandpass(&mut self, sr: f64, cutoff: f64, q: f64) {
        let cutoff = cutoff.clamp(20.0, sr * 0.45);
        let omega = TAU * cutoff / sr;
        let (so, co) = omega.sin_cos();
        let alpha = so / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        self.b0 = alpha / a0;
        self.b1 = 0.0;
        self.b2 = -alpha / a0;
        self.a1 = (-2.0 * co) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Highpass — used to keep the snare wires above the body.
    pub fn highpass(&mut self, sr: f64, cutoff: f64, q: f64) {
        let cutoff = cutoff.clamp(20.0, sr * 0.45);
        let omega = TAU * cutoff / sr;
        let (so, co) = omega.sin_cos();
        let alpha = so / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + co) * 0.5) / a0;
        self.b1 = -(1.0 + co) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * co) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Reset filter state (history) without changing coefficients. Call at note start.
    #[inline]
    pub fn reset_state(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    /// Direct Form II Transposed processing — numerically stable, single-sample.
    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TPT STATE-VARIABLE FILTER — topology-preserving zero-delay design
//
// Unlike a fixed biquad, this filter's cutoff can be modulated per-sample
// without aliasing or instability. That matters because the *signature*
// sound of an analog drum machine is a filter whose cutoff sweeps with the
// envelope (kick body darkens as it decays, snare shell loses brightness).
// Static filters can't do that — they sound like a synth, not an instrument.
// Reference: Vadim Zavalishin, "The Art of VA Filter Design," chapter 5.
// ═══════════════════════════════════════════════════════════════════════
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TptSvf {
    ic1eq: f64,
    ic2eq: f64,
}

impl TptSvf {
    #[inline]
    pub fn reset_state(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Process one sample as a resonant lowpass.
    /// `cutoff` and `q` may change every call without artifacts.
    #[inline]
    pub fn process_lp(&mut self, x: f64, sr: f64, cutoff: f64, q: f64) -> f64 {
        let g = (PI * cutoff.clamp(15.0, sr * 0.49) / sr).tan();
        let k = 1.0 / q.max(0.1);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let v3 = x - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + g * v1;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v2
    }

    /// Process one sample as a resonant bandpass.
    #[inline]
    pub fn process_bp(&mut self, x: f64, sr: f64, cutoff: f64, q: f64) -> f64 {
        let g = (PI * cutoff.clamp(15.0, sr * 0.49) / sr).tan();
        let k = 1.0 / q.max(0.1);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let v3 = x - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + g * v1;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v1
    }

    /// Process one sample as a resonant highpass.
    #[inline]
    pub fn process_hp(&mut self, x: f64, sr: f64, cutoff: f64, q: f64) -> f64 {
        let g = (PI * cutoff.clamp(15.0, sr * 0.49) / sr).tan();
        let k = 1.0 / q.max(0.1);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let v3 = x - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + g * v1;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        x - k * v1 - v2
    }
}

/// Asymmetric soft saturation — different bend on the positive and negative
/// half-cycles, like an analog tube or transformer. Adds even-order harmonics
/// (2nd, 4th) which read as "warmth" / "body" instead of the harsh-edged
/// odd-only harmonics of symmetric tanh.
#[inline]
fn asym_saturate(x: f64, drive: f64, asymmetry: f64) -> f64 {
    let d = x * drive;
    // Bias the input — `asymmetry` shifts the operating point so the
    // positive and negative halves bend through different parts of the
    // tanh curve. The DC removal afterwards keeps the output centred.
    let biased = d + asymmetry;
    let shaped = biased.tanh();
    let dc = asymmetry.tanh();
    shaped - dc
}

/// One-pole envelope follower with asymmetric attack/release for compression.
/// Stateless interface — call with previous env state, get back the new one.
#[inline]
fn peak_follow(prev_env: f64, x: f64, atk_coef: f64, rel_coef: f64) -> f64 {
    let abs_x = x.abs();
    let coef = if abs_x > prev_env { atk_coef } else { rel_coef };
    prev_env + coef * (abs_x - prev_env)
}

/// Soft-knee compressor curve. Approximates a real compressor's gain
/// reduction without per-sample log/exp. Returns the gain multiplier.
/// `env`: detector level. `threshold`: where compression starts (linear).
/// `ratio`: 1.0 = none, 4.0 = 4:1, etc.
#[inline]
fn compress_gain(env: f64, threshold: f64, ratio: f64) -> f64 {
    let over = env - threshold;
    if over <= 0.0 {
        1.0
    } else {
        // Smooth gain reduction that approaches 1/ratio asymptotically.
        // For small `over`, it's nearly linear pass-through; for large
        // `over`, it pulls the signal toward threshold at the ratio.
        1.0 / (1.0 + over * (ratio - 1.0))
    }
}

/// Per-voice drum filter state. One block holds every filter any drum needs;
/// unused slots are free (a few floats). Reset and re-coefficented at the
/// start of each drum note in `Voice::generate_waveform`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrumState {
    // Kick — TPT SVF so we can modulate the lowpass cutoff with the pitch
    // envelope. As the kick body decays, the lowpass drops in cutoff.
    pub kick_body: TptSvf,
    // Bandpass for the kick click — excited by a brief impulse + noise burst
    // at note start. The filter's impulse response IS the click, so it has
    // natural chaotic ring instead of a smooth synthesized envelope.
    pub kick_click_bp: Biquad,
    // Per-drum compression envelope follower (parallel comp on the final mix).
    pub kick_comp_env: f64,
    // Snare wires bandpass cascade (fixed cutoff — wire metal resonance is
    // a fixed mechanical property).
    pub snare_wires: Biquad,
    pub snare_wires_2: Biquad,
    // Shell lowpass — modulated cutoff so the snare darkens as it decays.
    pub snare_shell_lp: TptSvf,
    // Beater bandpass — drives the initial stick impact. Excited by a brief
    // noise burst at note start; filter ring gives a tonal "thwack."
    pub snare_beater_bp: Biquad,
    // Snare compression envelope.
    pub snare_comp_env: f64,
    // HiHat bandpasses on noise; TPT highpass on modes.
    pub hihat_bp_low: Biquad,
    pub hihat_bp_high: Biquad,
    pub hihat_hp: TptSvf,
    // Modal coupling feedback for the hi-hat.
    pub coupling_z: f64,
    // Per-note seed for phase randomization. Increments at each reset(),
    // mixed into each mode's starting phase so no two hits are byte-identical.
    // Without this, every drum hit has exactly the same waveform — the #1
    // perceptual "synth tell" that the brain catches instantly.
    pub note_counter: u32,
}

impl DrumState {
    pub fn reset(&mut self) {
        self.kick_body.reset_state();
        self.kick_click_bp.reset_state();
        self.kick_comp_env = 0.0;
        self.snare_wires.reset_state();
        self.snare_wires_2.reset_state();
        self.snare_shell_lp.reset_state();
        self.snare_beater_bp.reset_state();
        self.snare_comp_env = 0.0;
        self.hihat_bp_low.reset_state();
        self.hihat_bp_high.reset_state();
        self.hihat_hp.reset_state();
        self.coupling_z = 0.0;
        self.note_counter = self.note_counter.wrapping_add(1);
    }

    /// Generate a small phase offset (in radians) deterministic per-note,
    /// per-filter-id. Two different ids on the same note get different
    /// offsets; the same id on different notes gets different offsets.
    /// Range ≈ [-π/30, +π/30] — small enough to not change pitch perception,
    /// large enough to break the byte-identical-hit perceptual tell.
    #[inline]
    pub fn phase_jitter(&self, id: u64) -> f64 {
        let mut x = (self.note_counter as u64).wrapping_mul(0x9E3779B97F4A7C15);
        x = x.wrapping_add(id);
        x ^= x >> 33;
        x = x.wrapping_mul(0xff51afd7ed558ccd);
        x ^= x >> 33;
        let normalized = ((x as i64) as f64) / (i64::MAX as f64);  // [-1, 1]
        normalized * (PI / 30.0)
    }

    /// Return a u64 seed unique per-note for noise functions that want
    /// per-hit variation in their seed space.
    #[inline]
    pub fn noise_seed(&self, id: u64) -> u64 {
        let mut x = (self.note_counter as u64).wrapping_mul(0x9E3779B97F4A7C15);
        x ^= id;
        x ^= x >> 33;
        x.wrapping_mul(0xff51afd7ed558ccd)
    }
}

impl Voice {
    #[inline]
    pub fn calculate_current_phase(info: &SampleInfo, osc_type: &OscType, prev_phase: f64) -> f64 {
        let rand = if *osc_type == OscType::Noise {
            noise_offset(info.sample_index)
        } else {
            0.0
        };

        if info.gain == 0.0 {
            0.0
        } else {
            ((TAU / info.sample_rate).mul_add(info.frequency, prev_phase) + rand)
                % TAU
        }
    }
}
pub trait Waveform {
    /// `state` is per-voice drum filter state. Non-drum oscillators ignore it.
    fn generate_sample(&self, info: SampleInfo, phase: f64, state: &mut DrumState) -> f64;
}

impl Waveform for OscType {
    #[inline]
    fn generate_sample(&self, info: SampleInfo, phase: f64, state: &mut DrumState) -> f64 {
        match self {
            OscType::None => phase.sin() * info.gain,
            OscType::Sine { pow } => {
                let value = match pow {
                    Some(p) => {
                        let power = r_to_f64(*p);
                        f64::powf(phase, power).sin() / power
                    }
                    None => phase.sin(),
                };
                value * info.gain
            }
            OscType::Fm { defs } => {
                let carrier_freq = info.frequency;
                let rate_factor = TAU / info.sample_rate;

                let modulator_samples = defs
                    .iter()
                    .map(|def| {
                        let modulation_index = r_to_f64(def.depth);
                        let modulator_frequency_multiple = r_to_f64(def.fm);
                        let modulator_freq = carrier_freq * modulator_frequency_multiple;
                        let modulator_phase = (rate_factor.mul_add(modulator_freq, phase)) % TAU;

                        modulator_phase.sin() * modulation_index
                    })
                    .sum::<f64>();

                let carrier_phase =
                    (rate_factor.mul_add(carrier_freq, phase + modulator_samples)) % TAU;

                carrier_phase.sin() * info.gain
            }
            OscType::Triangle { pow } => {
                let value = match pow {
                    Some(p) => {
                        let power = r_to_f64(*p);
                        (f64::powf(phase, power).sin().abs() * 2.0 - 1.0) / power
                    }
                    None => phase.sin().abs() * 2.0 - 1.0,
                };
                value * info.gain
            }
            OscType::Square { width } => {
                let pulse_width = width.map_or(0.0, r_to_f64);
                let sign = if phase.sin() > pulse_width { -1. } else { 1. };
                sign * info.gain
            }
            OscType::Saw => 2.0 * (phase / TAU - 0.5_f64.floor()) * info.gain,
            OscType::Noise => phase.sin() * info.gain,

            // ═════════════════════════════════════════════════════════════════════
            // KICK — 808-flavoured drum synthesis
            //
            // Layers (in time order they peak):
            //   1. Beater noise burst        (~0-3 ms)   — broadband transient
            //   2. Click tone                (~0-5 ms)   — sine in 1.5-3 kHz
            //   3. Pitch-swept sine body     (~0-50 ms sweep, 200ms-1s decay)
            //   4. Self-FM warmth during sweep — adds harmonic edge so the
            //      transient isn't just a pure sine doing a glide.
            //   5. Tanh saturation on the whole thing — fattens the body,
            //      adds odd harmonics, glues click to body.
            //
            // The "punch hump" is a brief level pump on the body during the
            // first 8 ms — real 808 kicks have this from the way the envelope
            // ramp interacts with the analog VCA, and it's the single biggest
            // psychoacoustic cue for "this kick hits hard".
            // ═════════════════════════════════════════════════════════════════════
            OscType::Kick { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let attack_spec = params.as_ref().and_then(|p| p.attack.map(r_to_f64)).unwrap_or(0.5);
                let body_spec = params.as_ref().and_then(|p| p.body.map(r_to_f64)).unwrap_or(0.5);
                let tone_spec = params.as_ref().and_then(|p| p.tone.map(r_to_f64)).unwrap_or(0.5);
                let length_spec = params.as_ref().and_then(|p| p.length.map(r_to_f64)).unwrap_or(0.5);

                let tune = params.as_ref().and_then(|p| p.tune.map(r_to_f64)).unwrap_or(1.0);
                let info_freq = if info.frequency > 20.0 { info.frequency } else { KICK_DEFAULT_FREQ };
                let f_base = info_freq * tune;

                let pitch_decay = params.as_ref()
                    .and_then(|p| p.pitch_decay.map(r_to_f64))
                    .unwrap_or(0.045);
                let pitch_range = params.as_ref()
                    .and_then(|p| p.pitch_range.map(r_to_f64))
                    .unwrap_or(4.5);
                let amp_decay = params.as_ref()
                    .and_then(|p| p.amp_decay.map(r_to_f64))
                    .unwrap_or(0.35 + length_spec * 0.55);
                let click_amount = params.as_ref()
                    .and_then(|p| p.click_amount.map(r_to_f64))
                    .unwrap_or(0.10 + attack_spec * 0.35);
                let click_freq = params.as_ref()
                    .and_then(|p| p.click_freq.map(r_to_f64))
                    .unwrap_or(1700.0 + tone_spec * 1300.0);  // 1.7–3.0 kHz
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(0.40 + body_spec * 0.30);
                // `hump` is the kick's punch knob — depth of the initial level
                // pump on the body. Default 0.5 gives a clearly perceptible
                // "thump" without making the kick distort.
                let hump_amount = params.as_ref()
                    .and_then(|p| p.hump.map(r_to_f64))
                    .unwrap_or(0.5);

                let velocity = info.gain.clamp(0.0, 1.0);
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.5);
                let spectral_tilt = velocity.sqrt();
                let click_amount_vel = click_amount * (0.6 + 0.4 * spectral_tilt * velocity_tilt);

                // ─── Per-note setup: click bandpass tuned to click_freq ──────
                // High-Q bandpass excited by a brief impulse gives us the
                // click — the filter's impulse response IS the transient,
                // which naturally has the chaotic ring of a real beater.
                if info.sample_index == 0 {
                    state.kick_click_bp.bandpass(info.sample_rate, click_freq, 6.0);
                }

                // ─── Pitch-envelope phase integration ─────────────────────────
                //   freq(t) = f_base * (1 + (R-1) * exp(-t/τ))
                //   τ = pitch_decay / 3 → ~95% settled at t = pitch_decay
                let tau_p = pitch_decay / 3.0;
                let exp_pd = (-t / tau_p).exp();
                let env_factor = (pitch_range - 1.0) * tau_p * (1.0 - exp_pd);
                // Phase jitter — per-note tiny offset so hits aren't identical.
                let jitter = state.phase_jitter(0x1CC0_F00D);
                let kick_phase = TAU * f_base * (t + env_factor) + jitter;

                // ─── Self-FM during the pitch sweep ───────────────────────────
                // Modulate the kick by a 2nd-harmonic sine whose depth tracks
                // the pitch envelope. When the pitch is high (transient), the
                // FM is active and adds edge; once the body settles the FM
                // contribution vanishes and we're back to a pure sine.
                let fm_depth = 0.6 * exp_pd;          // peaks at t=0, gone by ~pitch_decay
                let fm_mod = (kick_phase * 2.0).sin() * fm_depth;
                let fundamental = (kick_phase + fm_mod).sin();

                // ─── Amplitude envelope with initial punch hump ───────────────
                // Body decay is a smooth exponential. On top of that, we add a
                // brief 8 ms hump that pumps the body up by `hump_amount`. The
                // hump uses a `t * exp(-t/τ_h)` shape (peaks at τ_h, decays
                // either side) — sounds like analog compression release.
                let body_decay_env = (-t * 3.0 / amp_decay).exp();
                let tau_h = 0.004;                    // 4 ms hump centre
                let hump = hump_amount * (t / tau_h) * (-t / tau_h).exp() * std::f64::consts::E;
                let body_env = body_decay_env * (1.0 + hump);

                // ─── Click — bandpass-filtered impulse + brief noise burst ────
                // Instead of synthesizing a smooth sine envelope, we feed a
                // short impulse + chaotic noise into a high-Q bandpass and
                // let the filter ring out at click_freq. The result is a
                // tonal "thwack" with natural ring decay — much closer to a
                // real beater hit than a clean sine envelope.
                let click_excite = if info.sample_index < 4 {
                    let ramp = 1.0 - info.sample_index as f64 * 0.25;
                    let chaos = fast_noise(info.sample_index, state.noise_seed(0xC11C_BEEF));
                    ramp + chaos * 0.5
                } else if t < 0.001 {
                    // Tiny continued noise excitation for the first ms,
                    // gives the click "beater chatter" character.
                    fast_noise(info.sample_index, state.noise_seed(0xC11C_FACE)) * 0.3
                } else {
                    0.0
                };
                let click_raw = state.kick_click_bp.process(click_excite);
                let click = click_raw * click_amount_vel * 4.5;

                // ─── Body: asymmetric saturation → modulated lowpass ──────────
                // Two analog moves stacked here:
                //
                // 1) Asymmetric saturation instead of plain tanh. Real tubes
                //    and transformers bend the positive and negative half-
                //    cycles through different parts of the curve, producing
                //    even-order harmonics (2nd, 4th) on top of the odd ones.
                //    Perceived as "warmth" and "body" rather than the harsher
                //    "fuzz" of pure odd-harmonic distortion.
                //
                // 2) The body lowpass cutoff TRACKS the pitch envelope. At
                //    t=0 the cutoff is high (lets the transient click and
                //    upper harmonics through); as the body decays the cutoff
                //    drops, naturally darkening the tail. This is the
                //    signature analog drum-machine sound — a static filter
                //    sounds like a synth; a sweeping filter sounds like an
                //    actual kick drum.
                let body = fundamental * body_env;
                let drive = 1.0 + saturation_amount * 3.0 * body_decay_env;
                let asym = saturation_amount * 0.4 * body_decay_env;
                let body_sat = asym_saturate(body, drive, asym) * (1.0 + saturation_amount * 0.5);

                // Cutoff sweeps from ~6× fundamental during the transient
                // down to ~1.5× as the body decays. Q stays near unity-and-
                // a-bit — enough resonance to add weight, not so much that
                // the kick whistles.
                let pitch_env = 1.0 + (pitch_range - 1.0) * exp_pd;        // (R..1)
                let cutoff_hi = f_base * 6.0 * pitch_env;
                let cutoff_lo = f_base * 1.5;
                let cutoff = cutoff_lo + (cutoff_hi - cutoff_lo) * body_decay_env;
                let body_shaped = state.kick_body.process_lp(body_sat, info.sample_rate, cutoff, 1.2);

                // ─── Parallel compression — lifts body, keeps transient ───────
                // Fast attack so we catch the kick's initial spike; medium
                // release so the body holds its energy through the decay.
                // Then mix the compressed signal parallel with the dry: the
                // dry preserves the transient shape, the compressed lifts the
                // sustain. This is the classic "drum bus" compression sound.
                let dry = body_shaped + click;
                let atk = 1.0 - (-1.0 / (info.sample_rate * 0.0008)).exp();   // 0.8 ms
                let rel = 1.0 - (-1.0 / (info.sample_rate * 0.080)).exp();    // 80 ms
                state.kick_comp_env = peak_follow(state.kick_comp_env, dry, atk, rel);
                let comp_gain = compress_gain(state.kick_comp_env, 0.25, 5.0);
                let punchy = dry * 0.55 + (dry * comp_gain) * 0.90;

                punchy * info.gain * KICK_GAIN
            }

            // ═════════════════════════════════════════════════════════════════════
            // SNARE — multi-mode drum + sympathetic wires + noise-burst crack
            //
            // Physical model in spirit:
            //   - Top head: 2 inharmonic modes (Bessel-like ratios), pitched
            //     by `info.frequency * tune`. Top head decays slowly.
            //   - Bottom head: 2 modes at `shell_tune` × top, decays faster.
            //   - Wires: bright bandpassed noise SHAPED by the head envelopes
            //     — sympathetic vibration is how real wires get triggered.
            //   - Crack: a true noise burst (not pure sines) at the attack,
            //     gives the "snap" without sounding like a beep.
            //   - Beater click: very brief comb-noise burst (~1 ms).
            //
            // Why noise-shaped-by-modes? Real snare wires don't ring on their
            // own — they're driven by the bottom head. So when the head
            // envelope falls, the wire energy falls with it. This is what
            // makes a great snare sound "alive" instead of static.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Snare { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let attack_spec = params.as_ref().and_then(|p| p.attack.map(r_to_f64)).unwrap_or(0.5);
                let wires_spec = params.as_ref().and_then(|p| p.wires.map(r_to_f64)).unwrap_or(0.5);
                let tone_spec = params.as_ref().and_then(|p| p.tone.map(r_to_f64)).unwrap_or(0.5);
                let length_spec = params.as_ref().and_then(|p| p.length.map(r_to_f64)).unwrap_or(0.5);

                let tune = params.as_ref().and_then(|p| p.tune.map(r_to_f64)).unwrap_or(1.0);
                let info_freq = if info.frequency > 20.0 { info.frequency } else { SNARE_DEFAULT_FREQ };
                let f_base = info_freq * tune;

                // Per-note setup: wire bandpasses (fixed metal resonance),
                // and a beater bandpass at 3.5 kHz with high Q. The beater
                // BP is excited by a brief noise burst at note start; its
                // ring is the "stick impact" tonal component.
                if info.sample_index == 0 {
                    let bright_shift = 1.0 + tone_spec * 0.4;
                    state.snare_wires.bandpass(info.sample_rate, 5500.0 * bright_shift, 3.5);
                    state.snare_wires_2.bandpass(info.sample_rate, 9000.0 * bright_shift, 2.8);
                    state.snare_beater_bp.bandpass(info.sample_rate, 3500.0, 5.0);
                }

                let shell_decay = params.as_ref()
                    .and_then(|p| p.shell_decay.or(p.tone_decay).map(r_to_f64))
                    .unwrap_or(0.12 + length_spec * 0.12);
                let wire_decay = params.as_ref()
                    .and_then(|p| p.wire_decay.or(p.noise_decay).map(r_to_f64))
                    .unwrap_or(0.14 + wires_spec * 0.16);
                let wire_mix = params.as_ref()
                    .and_then(|p| p.wire_mix.or(p.noise_mix).map(r_to_f64))
                    .unwrap_or(0.35 + wires_spec * 0.35);
                let shell_tune = params.as_ref()
                    .and_then(|p| p.shell_tune.map(r_to_f64))
                    .unwrap_or(1.74);  // close to first Bessel inharmonic ratio
                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack_amount.map(r_to_f64))
                    .unwrap_or(0.18 + attack_spec * 0.32);
                let shell_pitch_decay = params.as_ref()
                    .and_then(|p| p.shell_pitch_decay.map(r_to_f64))
                    .unwrap_or(40.0);
                let shell_pitch_range = params.as_ref()
                    .and_then(|p| p.shell_pitch_range.map(r_to_f64))
                    .unwrap_or(0.15 + attack_spec * 0.35);
                let head_damping_ratio = params.as_ref()
                    .and_then(|p| p.head_damping_ratio.map(r_to_f64))
                    .unwrap_or(1.8 - tone_spec * 0.7);
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(0.20);
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.5);
                let crack_amount = params.as_ref()
                    .and_then(|p| p.crack.map(r_to_f64))
                    .unwrap_or(0.2 + attack_spec * 0.4);

                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.sqrt();
                let wire_mix_vel = wire_mix * (0.7 + 0.6 * spectral_tilt * velocity_tilt);

                // ─── Membrane modes (top + bottom head) ───────────────────────
                // Inharmonic ratios approximate the first few modes of a
                // circular membrane. The top head carries the fundamental and
                // a brighter overtone; the bottom head sits an octave-ish above
                // and decays faster.
                let pitch_mult = 1.0 + shell_pitch_range * (-t * shell_pitch_decay).exp();
                let top_f1 = f_base * pitch_mult;
                let top_f2 = f_base * 1.59 * pitch_mult;          // (1,1) mode
                let bot_f1 = f_base * shell_tune * pitch_mult;
                let bot_f2 = f_base * shell_tune * 1.51 * pitch_mult;

                let top_amp1 = (-t * 3.0 / shell_decay).exp();
                let top_amp2 = (-t * 3.0 * 1.4 / shell_decay).exp();
                let bot_amp1 = (-t * 3.0 * head_damping_ratio / shell_decay).exp();
                let bot_amp2 = (-t * 3.0 * head_damping_ratio * 1.6 / shell_decay).exp();

                // Raw head — each mode gets a tiny per-note phase jitter so
                // repeated hits are not byte-identical (key to escaping the
                // "drum machine" tell). Then through the modulated shell LP.
                let j1 = state.phase_jitter(0x5_DEAD_0001);
                let j2 = state.phase_jitter(0x5_DEAD_0002);
                let j3 = state.phase_jitter(0x5_DEAD_0003);
                let j4 = state.phase_jitter(0x5_DEAD_0004);
                let head_raw = (TAU * top_f1 * t + j1).sin() * 0.42 * top_amp1
                             + (TAU * top_f2 * t + j2).sin() * 0.22 * top_amp2
                             + (TAU * bot_f1 * t + j3).sin() * 0.30 * bot_amp1
                             + (TAU * bot_f2 * t + j4).sin() * 0.16 * bot_amp2;
                let shell_cutoff_hi = (f_base * 8.0).clamp(800.0, 6000.0);
                let shell_cutoff_lo = (f_base * 2.5).clamp(300.0, 2500.0);
                let shell_cutoff = shell_cutoff_lo + (shell_cutoff_hi - shell_cutoff_lo) * top_amp1;
                let head = state.snare_shell_lp.process_lp(head_raw, info.sample_rate, shell_cutoff, 1.5);

                // ─── Wires: white noise → resonant bandpass cascade ───────────
                // Same as before, but noise seed is per-note via state.noise_seed.
                // The two bandpasses (5.5 kHz + 9 kHz) carve out the metallic
                // resonance from white noise.
                let white = fast_noise(info.sample_index, state.noise_seed(0x717E_5));
                let wires_filtered = state.snare_wires.process(white) * 2.2
                                   + state.snare_wires_2.process(white) * 1.6;
                let wire_attack = (-t * 3.0 / 0.006).exp();
                let wire_tail = (-t * 3.0 / wire_decay).exp();
                let sympathy = 0.4 + 0.6 * bot_amp1;
                let wire_env = (wire_attack * 0.7 + wire_tail * 0.3) * sympathy;
                let wires = wires_filtered * wire_env;

                // ─── Crack: noise burst with per-note seed variation ──────────
                // Two stages: a sub-ms spike (comb-shaped around 4-6 kHz) and
                // a slightly slower body. The per-note noise seed means
                // consecutive snare hits have DIFFERENT noise patterns —
                // critical for breaking the "drum machine" perceptual tell.
                let crack_spike_env = (-t / 0.0010).exp();
                let crack_body_env = (-t / 0.0050).exp();
                let crack_seed = state.noise_seed(0xC4AC_BEEF);
                let crack_spike = (comb_noise(info.sample_index, crack_seed, 6) * 1.4
                                +  bright_noise(info.sample_index, crack_seed) * 0.6)
                                * crack_spike_env;
                let crack_body = comb_noise(info.sample_index,
                                             state.noise_seed(0xC4AC_FACE), 10) * crack_body_env;
                let crack = (crack_spike + crack_body * 0.5) * crack_amount * 1.6;

                // ─── Beater impact — bandpass filter excited by noise burst ───
                // The high-Q bandpass at 3.5 kHz rings briefly when excited;
                // we excite it with a sub-millisecond noise burst so the
                // ring carries the chaotic stick character. Much more like
                // a real beater than a bare bright-noise envelope.
                let beater_excite = if info.sample_index < 6 {
                    fast_noise(info.sample_index, state.noise_seed(0xBEAD_F00D)) * 1.2
                } else {
                    0.0
                };
                let beater_raw = state.snare_beater_bp.process(beater_excite);
                let beater = beater_raw * attack_amount * 4.0;

                // ─── Mix + saturation + parallel compression ─────────────────
                let head_component = head * (1.0 - wire_mix_vel * 0.35);
                let wire_component = wires * (0.4 + wire_mix_vel * 0.9);
                let tone = head_component + wire_component + crack + beater;

                let decay_progress = 1.0 - top_amp1;
                let drive = 1.0 + saturation_amount * (1.0 + decay_progress);
                let saturated = soft_saturate(tone, drive);

                // Parallel comp — fast attack to catch the crack peak, medium
                // release so the wire/body sustains push through.
                let atk = 1.0 - (-1.0 / (info.sample_rate * 0.0005)).exp();   // 0.5 ms
                let rel = 1.0 - (-1.0 / (info.sample_rate * 0.060)).exp();    // 60 ms
                state.snare_comp_env = peak_follow(state.snare_comp_env, saturated, atk, rel);
                let comp_gain = compress_gain(state.snare_comp_env, 0.30, 4.5);
                let punchy = saturated * 0.55 + (saturated * comp_gain) * 0.90;

                punchy * info.gain * SNARE_GAIN
            }

            // ═════════════════════════════════════════════════════════════════════
            // HIHAT — dense modal cymbal model + bandpassed metallic noise
            //
            // Real cymbals have *dozens* of significant modes at inharmonic
            // ratios (Chladni patterns on a circular plate). Six sines, like
            // the previous version, sounds like a tuned bell — not a hihat.
            // This version uses 10 modes at carefully chosen inharmonic ratios
            // (no integer relations), plus a comb-stage "metal noise" gated by
            // the modal envelope so the noise contribution decays *with* the
            // metal instead of leaking out behind it.
            //
            // The 10 ratios come from a stretched golden-mean inspired set
            // chosen so no two are within 5% of any integer ratio — this is
            // what creates the dense, atonal "tssssh" we associate with
            // cymbals.
            // ═════════════════════════════════════════════════════════════════════
            OscType::HiHat { open, params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let attack_spec = params.as_ref().and_then(|p| p.attack.map(r_to_f64)).unwrap_or(0.5);
                let metal_spec = params.as_ref().and_then(|p| p.metal.map(r_to_f64)).unwrap_or(0.5);
                let length_spec = params.as_ref().and_then(|p| p.length.map(r_to_f64)).unwrap_or(0.5);

                let tune = params.as_ref().and_then(|p| p.tune.map(r_to_f64)).unwrap_or(1.0);
                let info_freq = if info.frequency > 20.0 { info.frequency } else { HIHAT_DEFAULT_FREQ };

                // Decay default: open hats ring much longer than closed.
                let decay_default_closed = 30.0;
                let decay_default_open = 6.0;
                let decay_base = if *open { decay_default_open } else { decay_default_closed };
                let decay_range = if *open { 4.0 } else { 18.0 };
                let decay_rate = params.as_ref()
                    .and_then(|p| p.decay_rate.map(r_to_f64))
                    .unwrap_or(decay_base + (1.0 - length_spec) * decay_range);

                let shimmer_mult = params.as_ref()
                    .and_then(|p| p.shimmer.map(r_to_f64))
                    .unwrap_or(0.9 + metal_spec * 0.4);
                let brightness = params.as_ref()
                    .and_then(|p| p.brightness.map(r_to_f64))
                    .unwrap_or(0.5 + metal_spec * 0.6);
                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack_amount.map(r_to_f64))
                    .unwrap_or(0.15 + attack_spec * 0.35);
                let pitch_drop = params.as_ref()
                    .and_then(|p| p.pitch_drop.map(r_to_f64))
                    .unwrap_or(0.008 + attack_spec * 0.016);
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.4);

                // Bandpass cascade on noise = metallic "tssh." The mode
                // highpass uses TPT SVF; we don't sweep it (cymbals don't),
                // but the TPT version is more numerically robust for the
                // very high cutoff we run.
                if info.sample_index == 0 {
                    state.hihat_bp_low.bandpass(info.sample_rate, 6000.0, 4.0);
                    state.hihat_bp_high.bandpass(info.sample_rate, 11000.0, 3.0);
                    state.coupling_z = 0.0;
                }

                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.sqrt();
                let brightness_vel = brightness * (0.7 + 0.6 * spectral_tilt * velocity_tilt);

                // Slight pitch droop — cymbals lose high-end energy first.
                let pitch_drop_mult = 1.0 + pitch_drop * (-t * 6.0).exp();
                let base_freq = info_freq * tune * shimmer_mult * pitch_drop_mult;

                // 22-mode cymbal — first 22 Bessel-function zeros for a circular
                // plate (the actual physics of cymbal vibration), with three
                // intentional close pairs to create *beating*. Real cymbals have
                // 50-100 modes; 22 is plenty for perceptual realism while
                // keeping the compute reasonable.
                //
                // Each mode also receives a tiny phase modulation from
                // `coupling_z` — the sum of the previous sample's mode outputs.
                // This is the nonlinear coupling that makes modes "talk to each
                // other" and creates the alive, shimmering character of real
                // metal instead of a static stacked-sines chord.
                let modes: [(f64, f64, f64); 22] = [
                    (1.000, 0.14, 1.0),
                    (1.594, 0.18, 1.3),
                    (1.612, 0.13, 1.35),     // beats with 1.594
                    (2.135, 0.16, 1.7),
                    (2.295, 0.18, 1.9),
                    (2.310, 0.12, 1.95),     // beats with 2.295
                    (2.653, 0.15, 2.3),
                    (2.917, 0.14, 2.7),
                    (3.156, 0.13 * brightness_vel, 3.1),
                    (3.500, 0.11 * brightness_vel, 3.6),
                    (3.598, 0.09 * brightness_vel, 3.7),  // beats with 3.500
                    (3.652, 0.08 * brightness_vel, 3.8),  // beats with 3.598
                    (4.060, 0.09 * brightness_vel, 4.4),
                    (4.131, 0.07 * brightness_vel, 4.5),  // beats with 4.060
                    (4.601, 0.08 * brightness_vel, 5.2),
                    (4.832, 0.07 * brightness_vel, 5.5),
                    (5.158, 0.06 * brightness_vel, 6.0),
                    (5.412, 0.05 * brightness_vel, 6.5),
                    (5.872, 0.05 * brightness_vel, 7.0),
                    (6.205, 0.04 * brightness_vel, 7.7),
                    (6.560, 0.04 * brightness_vel, 8.4),
                    (6.957, 0.03 * brightness_vel, 9.2),
                ];

                // Modal coupling — strong drive (0.06) so modes audibly
                // interfere. Each mode also gets a per-note phase jitter
                // unique to its id, so repeated hi-hat hits don't replay
                // the exact same waveform.
                let coupling_drive = state.coupling_z * 0.06;

                let mut shimmer = 0.0;
                for (i, (ratio, amp, decay_mult)) in modes.iter().copied().enumerate() {
                    let mode_freq = base_freq * ratio;
                    if mode_freq >= info.sample_rate * 0.5 { continue; }
                    let mode_amp = (-t * decay_rate * decay_mult).exp();
                    let j = state.phase_jitter(0xCAFE_0000 ^ (i as u64).wrapping_mul(0x9E37));
                    let phase = TAU * mode_freq * t + coupling_drive + j;
                    shimmer += phase.sin() * amp * mode_amp;
                }
                state.coupling_z = shimmer;

                // Highpass at 1.5 kHz keeps modes out of the kick band.
                let shimmer_hp = state.hihat_hp.process_hp(shimmer, info.sample_rate, 1500.0, 0.7);

                // ─── Air — white noise → resonant bandpass cascade ────────────
                // Per-note noise seed via state.noise_seed — consecutive hi-hat
                // hits get different noise content, which is the difference
                // between "drum machine" and "actual instrument."
                let white = fast_noise(info.sample_index, state.noise_seed(0xCAFE_A11));
                let air = state.hihat_bp_low.process(white) * 1.6
                        + state.hihat_bp_high.process(white) * 1.2;
                let air_env = (-t * decay_rate * 1.1).exp();
                let air_signal = air * air_env;

                // ─── Sharp attack — stick contact ─────────────────────────────
                let attack_env = (-t / 0.0015).exp();
                let attack_noise =
                    bright_noise(info.sample_index, 0x1234567890ABCDEF) * attack_env * attack_amount;

                let tone = shimmer_hp * 0.45 + air_signal * 0.55 + attack_noise;

                tone * info.gain * HIHAT_GAIN
            }
        }
    }
}
