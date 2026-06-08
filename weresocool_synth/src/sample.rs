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

// ═══════════════════════════════════════════════════════════════════════
// KARPLUS-STRONG WAVEGUIDE
//
// A delay line + feedback lowpass + soft saturation in the loop. Excited
// at note start with a brief noise burst; the delay length determines the
// frequency, the loop gain controls decay. This is *physical modeling* —
// the audio comes from a self-oscillating energy loop, not an additive sum
// of sines. The result has the natural shape of a vibrating membrane:
// chaotic onset that settles into a tonal body, with characteristic
// harmonic content emerging from the feedback nonlinearity.
//
// We support fractional delay length so the pitch can sweep continuously
// (matches the existing pitch envelope). Delay length is `sr / freq_hz`.
// ═══════════════════════════════════════════════════════════════════════
#[derive(Clone, Debug, PartialEq)]
pub struct KarplusStrong {
    buffer: Vec<f64>,
    write_pos: usize,
    lp_z: f64,      // one-pole lowpass state for the feedback filter
}

impl Default for KarplusStrong {
    fn default() -> Self {
        // Sized for fundamentals as low as 20 Hz at 96 kHz (4800 samples).
        // Cheap memory; means we never have to resize at runtime.
        Self { buffer: vec![0.0; 5000], write_pos: 0, lp_z: 0.0 }
    }
}

impl KarplusStrong {
    pub fn reset_state(&mut self) {
        for s in self.buffer.iter_mut() { *s = 0.0; }
        self.write_pos = 0;
        self.lp_z = 0.0;
    }

    /// Inject an excitation sample (additive — accumulates with existing
    /// loop content). Use to drive the line with a noise burst at note
    /// start before processing samples.
    #[inline]
    pub fn excite(&mut self, x: f64) {
        // Write into the current write position WITHOUT advancing; this
        // ensures excitation lands on the same sample slot the next
        // `process()` call will write into for feedback.
        let len = self.buffer.len();
        let pos = self.write_pos % len;
        self.buffer[pos] += x;
    }

    /// Process one sample. `delay_samples` is the fractional delay length;
    /// `loop_gain` controls decay (0.85-0.99 typical for drums); `lp_coef`
    /// is the feedback lowpass mix (0.0 = bypass, 0.5 = standard KS).
    #[inline]
    pub fn process(&mut self, delay_samples: f64, loop_gain: f64, lp_coef: f64) -> f64 {
        let len = self.buffer.len();
        let len_f = len as f64;

        // Fractional read position via linear interp between two taps.
        let target = (self.write_pos as f64 + len_f - delay_samples.clamp(2.0, len_f - 2.0)) % len_f;
        let idx0 = target.floor() as usize % len;
        let idx1 = (idx0 + 1) % len;
        let frac = target - target.floor();
        let delayed = self.buffer[idx0] * (1.0 - frac) + self.buffer[idx1] * frac;

        // One-pole lowpass in the feedback path. Damps high-frequency content
        // each pass — what gives real strings/drums their "settling" toward
        // pure tone.
        self.lp_z = lp_coef * self.lp_z + (1.0 - lp_coef) * delayed;

        // Soft saturation in the loop adds harmonic richness and stabilises
        // the energy (prevents blowup if loop_gain is briefly > 1).
        let feedback = (self.lp_z * loop_gain).tanh() * 0.95;

        // Write feedback into the line; excitation may have already been
        // accumulated into this slot via excite().
        self.buffer[self.write_pos % len] += feedback;
        self.write_pos = (self.write_pos + 1) % len;

        delayed
    }
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
        1.0 / (1.0 + over * (ratio - 1.0))
    }
}

/// Velocity → "aggression" mapping. Smoothstep curve: ghosts stay
/// clearly below normal, normal hits sit near the middle, loud hits ramp
/// up dramatically. This is the curve that makes a real drummer's
/// dynamics translate — soft hits feel round and contained, hard hits
/// feel attacked and crushed, normal hits in between.
///   v=0.25 → 0.16   (ghost)
///   v=0.50 → 0.50   (normal)
///   v=0.75 → 0.84   (accent)
///   v=1.00 → 1.00   (rim shot)
#[inline]
fn vel_curve(v: f64) -> f64 {
    let n = v.clamp(0.0, 1.0);
    n * n * (3.0 - 2.0 * n)
}

/// Asymmetric tape-style soft clipper. Bends positive and negative half-
/// cycles through different parts of the curve — adds even-order harmonics
/// (warmth) on top of odd ones (drive). Applied at each drum's final
/// output stage; gives the kit the "channel-strip glue" of analog tape
/// without any state. Very gentle — meant to colour, not crush.
#[inline]
fn tape_sat(x: f64) -> f64 {
    let asym = 0.04;
    let driven = (x + asym) * 0.85;
    let shaped = driven / (1.0 + driven.abs() * 0.6);
    let dc = (asym * 0.85) / (1.0 + (asym * 0.85).abs() * 0.6);
    (shaped - dc) * 1.15
}

/// Per-voice drum filter state. One block holds every filter any drum needs;
/// unused slots are free (a few floats). Reset and re-coefficented at the
/// start of each drum note in `Voice::generate_waveform`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrumState {
    // Kick — TPT SVF so we can modulate the lowpass cutoff with the pitch
    // envelope. As the kick body decays, the lowpass drops in cutoff.
    pub kick_body: TptSvf,
    // KARPLUS-STRONG WAVEGUIDE — physical-model body for the kick. Replaces
    // the additive sine fundamental with a self-oscillating delay loop that
    // settles from chaos into tone, exactly the way a vibrating membrane
    // does. Excited at note start; the loop carries the body decay.
    pub kick_ks: KarplusStrong,
    // Bandpass for the kick click — excited by a brief impulse + noise burst
    // at note start. The filter's impulse response IS the click, so it has
    // natural chaotic ring instead of a smooth synthesized envelope.
    pub kick_click_bp: Biquad,
    // SHELL RESONATOR — narrow bandpass at ~110 Hz excited at note start.
    pub kick_shell_bp: Biquad,
    // Per-drum compression envelope follower (parallel comp on the final mix).
    pub kick_comp_env: f64,
    // SNARE WIRES — four bandpasses tuned to the actual resonance peaks
    // of stretched metallic wire material. Real wires aren't a smooth
    // bandpass; they have specific spectral fingerprints. Tapering Q/gain
    // matches measured snare-wire spectra.
    pub snare_wires: Biquad,
    pub snare_wires_2: Biquad,
    pub snare_wires_3: Biquad,
    pub snare_wires_4: Biquad,
    // Shell lowpass — modulated cutoff so the snare darkens as it decays.
    pub snare_shell_lp: TptSvf,
    // Snare KS waveguide — physical-model body for the snare. Adds the same
    // organic-membrane character we have on the kick. Excited at note start,
    // delay length matches the snare fundamental (top_f1).
    pub snare_ks: KarplusStrong,
    // Beater bandpass — drives the initial stick impact. Excited by a brief
    // noise burst at note start; filter ring gives a tonal "thwack."
    pub snare_beater_bp: Biquad,
    // MID-BAND CRACK TAP — extracts 2-5 kHz of the snare tone for
    // aggressive waveshaping. This is where real snares get their "bite":
    // the membrane wrinkles at high amplitude and produces chaotic harmonic
    // distortion specifically in this band. Saturating only this slice and
    // mixing it back leaves the body clean while adding the characteristic
    // mid-band aggression.
    pub snare_crack_bp: Biquad,
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
    pub note_counter: u32,
    // Voice index (0 = L, 1 = R). Set once at Voice init and mixed into all
    // seeds. This is what gives the kit a STEREO IMAGE.
    pub voice_index: u32,
    // HAAS DELAY — per-voice sample offset added to noise reads. Right
    // channel's noise lags the left by 3 samples (~62 µs at 48 kHz); the
    // brain reads inter-channel time differences in this range as spatial
    // direction, not as echo. Combined with the per-voice noise seeds, this
    // gives the cymbal & wire content a strong spatial spread.
    pub noise_delay_samples: u32,
}

impl DrumState {
    pub fn reset(&mut self) {
        self.kick_body.reset_state();
        self.kick_ks.reset_state();
        self.kick_click_bp.reset_state();
        self.kick_shell_bp.reset_state();
        self.kick_comp_env = 0.0;
        self.snare_wires.reset_state();
        self.snare_wires_2.reset_state();
        self.snare_wires_3.reset_state();
        self.snare_wires_4.reset_state();
        self.snare_shell_lp.reset_state();
        self.snare_ks.reset_state();
        self.snare_beater_bp.reset_state();
        self.snare_crack_bp.reset_state();
        self.snare_comp_env = 0.0;
        self.hihat_bp_low.reset_state();
        self.hihat_bp_high.reset_state();
        self.hihat_hp.reset_state();
        self.coupling_z = 0.0;
        self.note_counter = self.note_counter.wrapping_add(1);
    }

    /// Generate a small phase offset (in radians) deterministic per-note,
    /// per-filter-id, and per-voice (L/R). The voice_index contribution is
    /// what gives the kit stereo width — left and right voices return
    /// different jitters for the same id+note. Range ≈ [-π/15, +π/15].
    #[inline]
    pub fn phase_jitter(&self, id: u64) -> f64 {
        let mut x = (self.note_counter as u64).wrapping_mul(0x9E3779B97F4A7C15);
        x = x.wrapping_add(id);
        x = x.wrapping_add((self.voice_index as u64).wrapping_mul(0xBF58476D1CE4E5B9));
        x ^= x >> 33;
        x = x.wrapping_mul(0xff51afd7ed558ccd);
        x ^= x >> 33;
        let normalized = ((x as i64) as f64) / (i64::MAX as f64);  // [-1, 1]
        normalized * (PI / 15.0)
    }

    /// Per-voice tiny frequency multiplier — sub-percent. Same f_base × this
    /// multiplier on L and R produces a slow beating that the ear reads as
    /// "wide stereo" without any perceptible detuning. Range ≈ ±0.3%.
    #[inline]
    pub fn freq_jitter(&self, id: u64) -> f64 {
        let mut x = (self.note_counter as u64).wrapping_mul(0xDA942042E4DD58B5);
        x = x.wrapping_add(id);
        x = x.wrapping_add((self.voice_index as u64).wrapping_mul(0xC2B2AE3D27D4EB4F));
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58476d1ce4e5b9);
        x ^= x >> 27;
        let normalized = ((x as i64) as f64) / (i64::MAX as f64);  // [-1, 1]
        1.0 + normalized * 0.003
    }

    /// Return a u64 seed unique per-note, per-id, per-voice. Different L/R
    /// seeds = different noise content per channel = natural stereo width
    /// on noise sources (wires, hat air, kick click chatter).
    #[inline]
    pub fn noise_seed(&self, id: u64) -> u64 {
        let mut x = (self.note_counter as u64).wrapping_mul(0x9E3779B97F4A7C15);
        x ^= id;
        x ^= (self.voice_index as u64).wrapping_mul(0x94D049BB133111EB);
        x ^= x >> 33;
        x.wrapping_mul(0xff51afd7ed558ccd)
    }

    /// Sample-shifted index for Haas-style stereo decorrelation. Right
    /// channel's noise lags the left by `noise_delay_samples` samples;
    /// since `fast_noise` is deterministic on index, this is a free delay
    /// line that produces inter-channel time difference without ringing.
    #[inline]
    pub fn haas_index(&self, sample_index: usize) -> usize {
        sample_index.saturating_sub(self.noise_delay_samples as usize)
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
                let shell_amount = params.as_ref()
                    .and_then(|p| p.shell.map(r_to_f64))
                    .unwrap_or(0.45 + body_spec * 0.20);
                let ks_mix = params.as_ref()
                    .and_then(|p| p.ks_mix.map(r_to_f64))
                    .unwrap_or(0.85);

                let velocity = info.gain.clamp(0.0, 1.0);
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.5);
                let spectral_tilt = velocity.sqrt();
                // EXPRESSIVE VELOCITY: aggressive curve scales click, saturation,
                // and the body-LFO depth. Soft hits = rounded, no click, lots of
                // breath. Hard hits = saturated, click-heavy, punchy with no LFO.
                let aggression = vel_curve(velocity);
                let click_amount_vel = click_amount * (0.25 + 1.05 * aggression * velocity_tilt);
                let sat_vel = 0.45 + 0.7 * aggression;       // 0.45-1.15× saturation
                let lfo_vel = 1.0 - 0.85 * aggression;       // soft hits breathe, hard hits don't

                // ─── Per-note setup ─────────────────────────────────────────
                if info.sample_index == 0 {
                    state.kick_click_bp.bandpass(info.sample_rate, click_freq, 6.0);
                    let shell_hz = (f_base * 1.78 + 18.0).clamp(60.0, 220.0);
                    state.kick_shell_bp.bandpass(info.sample_rate, shell_hz, 5.0);
                }
                // KS excitation: noise burst into the waveguide loop.
                // CRITICAL: use a voice-independent seed so L and R produce
                // identical kick bodies — the kick is the LOW-FREQUENCY
                // anchor of the mix and decorrelating its body would kill
                // bass translation on mono speakers. The note_counter still
                // varies per hit so consecutive kicks are not identical;
                // we just want L = R within a single hit for the body.
                if t < 0.003 {
                    let mono_seed = (state.note_counter as u64)
                        .wrapping_mul(0xCAFE_BABE_DEAD_BEEF);
                    let burst = fast_noise(info.sample_index, mono_seed);
                    state.kick_ks.excite(burst * 0.6);
                }

                // ─── Pitch-envelope phase integration (TWO-STAGE) ─────────────
                // Real 808/909 kicks have a piecewise pitch shape: a *very*
                // fast initial drop (the "knee") followed by a slower settle
                // to the fundamental. A pure single exponential sounds too
                // smooth. We build it as a weighted sum of two exponentials —
                // the fast one drops most of the way in ~30% of pitch_decay,
                // the slow one carries the remaining settle.
                let tau_fast = pitch_decay * 0.10;
                let tau_slow = pitch_decay / 3.0;
                let exp_fast = (-t / tau_fast).exp();
                let exp_slow = (-t / tau_slow).exp();
                // 65/35 weighting: the fast knee dominates the early shape.
                let exp_pd = 0.65 * exp_fast + 0.35 * exp_slow;
                // Closed-form integral of `f_base * (1 + (R-1) * exp_pd)`
                let integral_fast = tau_fast * (1.0 - exp_fast);
                let integral_slow = tau_slow * (1.0 - exp_slow);
                let env_factor = (pitch_range - 1.0)
                    * (0.65 * integral_fast + 0.35 * integral_slow);
                let jitter = state.phase_jitter(0x1CC0_F00D);
                let kick_phase = TAU * f_base * (t + env_factor) + jitter;

                // ─── KS body — physical-model waveguide ──────────────────────
                // The delay length sweeps with the pitch envelope: high
                // frequency (short delay) at the transient, settling to the
                // fundamental as the pitch envelope decays. The loop's feed-
                // back filter does the spectral darkening as it rings out —
                // no separate "lowpass on body" required.
                //
                // Loop gain derived so the waveguide energy reaches ~5% at
                // `amp_decay` seconds, independent of pitch. Subtle noise
                // injection keeps the loop alive past the initial burst.
                let freq_now = f_base * (1.0 + (pitch_range - 1.0) * exp_pd);
                let delay_samps = info.sample_rate / freq_now.max(20.0);
                let loop_gain = 0.05_f64.powf(delay_samps / (amp_decay * info.sample_rate))
                                  .clamp(0.80, 0.998);
                // LP coef: more damping as the body decays → settling sound.
                // Use the amp-decay envelope hoisted up from below.
                let body_decay_env = (-t * 3.0 / amp_decay).exp();
                let lp_coef = 0.30 + 0.25 * (1.0 - body_decay_env);

                // Additive sine still contributes — gives us a tunable,
                // predictable fundamental. KS layer adds physical-model
                // character on top. 50/50 blend by default.
                let fm_depth = 0.6 * exp_pd;
                let fm_mod = (kick_phase * 2.0).sin() * fm_depth;
                let sine_fundamental = (kick_phase + fm_mod).sin();
                let ks_voice = state.kick_ks.process(delay_samps, loop_gain, lp_coef);
                let fundamental = sine_fundamental * 0.55 + ks_voice * ks_mix;

                // ─── Amplitude envelope with initial punch hump + body LFO ────
                // body_decay_env was computed above for the KS lp_coef.
                let tau_h = 0.004;
                let hump = hump_amount * (t / tau_h) * (-t / tau_h).exp() * std::f64::consts::E;
                let lfo_phase = TAU * 7.0 * t + state.phase_jitter(0x1CC0_AAAA);
                let body_lfo = 1.0 + 0.04 * lfo_vel * lfo_phase.sin() * (1.0 - body_decay_env);
                let body_env = body_decay_env * (1.0 + hump) * body_lfo;

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
                // Velocity scales saturation: soft hits stay clean, hard hits
                // get pushed into asymmetric distortion territory.
                let sat_eff = saturation_amount * sat_vel;
                let drive = 1.0 + sat_eff * 3.0 * body_decay_env;
                let asym = sat_eff * 0.4 * body_decay_env;
                let body_sat = asym_saturate(body, drive, asym) * (1.0 + sat_eff * 0.5);

                // Cutoff sweeps from ~6× fundamental during the transient
                // down to ~1.5× as the body decays. Q stays near unity-and-
                // a-bit — enough resonance to add weight, not so much that
                // the kick whistles.
                let pitch_env = 1.0 + (pitch_range - 1.0) * exp_pd;        // (R..1)
                let cutoff_hi = f_base * 6.0 * pitch_env;
                let cutoff_lo = f_base * 1.5;
                let cutoff = cutoff_lo + (cutoff_hi - cutoff_lo) * body_decay_env;
                let body_shaped = state.kick_body.process_lp(body_sat, info.sample_rate, cutoff, 1.2);

                // ─── Shell resonance ──────────────────────────────────────────
                // Excite the narrow shell BP with a short noise burst at note
                // start; the filter rings at the shell frequency, giving the
                // kick a subtle "wooden body" underneath. Decays naturally
                // from the bandpass's own resonance, modulated down so it's
                // felt more than heard.
                let shell_excite = if info.sample_index < 8 {
                    fast_noise(info.sample_index, state.noise_seed(0x5BE11_F00D)) * 1.0
                } else {
                    0.0
                };
                let shell_ring = state.kick_shell_bp.process(shell_excite);
                // Slow exponential decay on top of the BP's natural ring.
                let shell_env = (-t * 3.0 / (amp_decay * 0.7)).exp();
                let shell = shell_ring * shell_env * shell_amount;

                // ─── Parallel compression — lifts body, keeps transient ───────
                // Fast attack so we catch the kick's initial spike; medium
                // release so the body holds its energy through the decay.
                // Then mix the compressed signal parallel with the dry: the
                // dry preserves the transient shape, the compressed lifts the
                // sustain. This is the classic "drum bus" compression sound.
                let dry = body_shaped + click + shell;
                let atk = 1.0 - (-1.0 / (info.sample_rate * 0.0008)).exp();   // 0.8 ms
                let rel = 1.0 - (-1.0 / (info.sample_rate * 0.080)).exp();    // 80 ms
                state.kick_comp_env = peak_follow(state.kick_comp_env, dry, atk, rel);
                let comp_gain = compress_gain(state.kick_comp_env, 0.25, 5.0);
                let punchy = dry * 0.55 + (dry * comp_gain) * 0.90;

                // Output stage: tape-style asymmetric soft clip for "glue."
                tape_sat(punchy) * info.gain * KICK_GAIN
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

                // Per-note setup. Wire spectrum is now modelled with FOUR
                // bandpasses tuned to the actual resonance peaks of stretched
                // metallic snare-wire material. Real wires aren't a smooth
                // hiss in a wide band — they have specific resonances at
                // roughly 5.8/9.7/13.3/16.2 kHz (the wire fundamental and
                // its inharmonic overtones, slightly stretched by tension).
                // Each band's Q tapers down with frequency (higher modes
                // damp faster) and amplitude tapers too, matching measured
                // snare spectra.
                let shell_decay = params.as_ref()
                    .and_then(|p| p.shell_decay.map(r_to_f64))
                    .unwrap_or(0.12 + length_spec * 0.12);
                let wire_decay = params.as_ref()
                    .and_then(|p| p.wire_decay.map(r_to_f64))
                    .unwrap_or(0.14 + wires_spec * 0.16);
                let wire_mix = params.as_ref()
                    .and_then(|p| p.wire_mix.map(r_to_f64))
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
                let crack_freq_hz = params.as_ref()
                    .and_then(|p| p.crack_freq.map(r_to_f64))
                    .unwrap_or(3000.0);
                let snare_ks_mix = params.as_ref()
                    .and_then(|p| p.ks_mix.map(r_to_f64))
                    .unwrap_or(0.55);

                // Per-note filter setup. All cutoffs/Qs are fixed for the
                // life of the note; bandpasses on the wires model the actual
                // resonance peaks of stretched metal wire material.
                if info.sample_index == 0 {
                    let bright_shift = 1.0 + tone_spec * 0.4;
                    state.snare_wires.bandpass(info.sample_rate, 5800.0 * bright_shift, 4.0);
                    state.snare_wires_2.bandpass(info.sample_rate, 9700.0 * bright_shift, 3.4);
                    state.snare_wires_3.bandpass(info.sample_rate, 13300.0 * bright_shift, 2.8);
                    state.snare_wires_4.bandpass(info.sample_rate, 16200.0 * bright_shift, 2.3);
                    state.snare_beater_bp.bandpass(info.sample_rate, 3500.0, 5.0);
                    state.snare_crack_bp.bandpass(info.sample_rate, crack_freq_hz, 1.4);
                }

                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.sqrt();
                let aggression = vel_curve(velocity);
                let wire_mix_vel = wire_mix * (0.55 + 0.85 * aggression * velocity_tilt);
                // Velocity-modulated drives. Soft hits = round, no crack;
                // hard hits = full crack, more saturation, more wire.
                let crack_vel = 0.30 + 1.30 * aggression;
                let beater_vel = 0.40 + 1.00 * aggression;
                let sat_vel = 0.45 + 0.80 * aggression;
                let mid_crack_vel = 0.20 + 1.30 * aggression;

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

                // Raw head — sine modes with phase + freq jitter for L/R width.
                let j1 = state.phase_jitter(0x5_DEAD_0001);
                let j2 = state.phase_jitter(0x5_DEAD_0002);
                let j3 = state.phase_jitter(0x5_DEAD_0003);
                let j4 = state.phase_jitter(0x5_DEAD_0004);
                let fj1 = state.freq_jitter(0x5_DEAD_0001);
                let fj2 = state.freq_jitter(0x5_DEAD_0002);
                let fj3 = state.freq_jitter(0x5_DEAD_0003);
                let fj4 = state.freq_jitter(0x5_DEAD_0004);
                let head_sines = (TAU * top_f1 * fj1 * t + j1).sin() * 0.36 * top_amp1
                               + (TAU * top_f2 * fj2 * t + j2).sin() * 0.20 * top_amp2
                               + (TAU * bot_f1 * fj3 * t + j3).sin() * 0.26 * bot_amp1
                               + (TAU * bot_f2 * fj4 * t + j4).sin() * 0.14 * bot_amp2;

                // ─── KS body — physical-model membrane character ─────────────
                // Voice-independent excitation seed so L and R produce the
                // same snare body — body is centered in the stereo image,
                // wires/crack/beater carry all the width. Just like the kick.
                if t < 0.003 {
                    let mono_seed = (state.note_counter as u64)
                        .wrapping_mul(0xBABE_F00D_5A11_5A11);
                    let burst = fast_noise(info.sample_index, mono_seed);
                    state.snare_ks.excite(burst * 0.5);
                }
                let snare_delay = info.sample_rate / (top_f1 * pitch_mult).max(40.0);
                let snare_lg = 0.05_f64
                    .powf(snare_delay / (shell_decay * info.sample_rate))
                    .clamp(0.70, 0.985);
                let snare_lp = 0.35 + 0.20 * (1.0 - top_amp1);
                let ks_body = state.snare_ks.process(snare_delay, snare_lg, snare_lp) * snare_ks_mix;
                let head_raw = head_sines + ks_body;
                let shell_cutoff_hi = (f_base * 8.0).clamp(800.0, 6000.0);
                let shell_cutoff_lo = (f_base * 2.5).clamp(300.0, 2500.0);
                let shell_cutoff = shell_cutoff_lo + (shell_cutoff_hi - shell_cutoff_lo) * top_amp1;
                let head = state.snare_shell_lp.process_lp(head_raw, info.sample_rate, shell_cutoff, 1.5);

                // ─── Wires: 4-band spectral envelope on white noise ───────────
                // Real snare wires don't have a smooth-bandpass spectrum;
                // they have peaks at specific resonance frequencies of the
                // wire material. Tapping all four bands and summing with
                // amplitude-weights matching measured spectra gives a much
                // more identifiable "snare wire" timbre than two BPs.
                // Per-note seed + Haas delay for stereo width.
                let white = fast_noise(state.haas_index(info.sample_index),
                                        state.noise_seed(0x717E_5));
                let wires_filtered = state.snare_wires.process(white) * 1.8
                                   + state.snare_wires_2.process(white) * 1.3
                                   + state.snare_wires_3.process(white) * 0.85
                                   + state.snare_wires_4.process(white) * 0.55;
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
                let crack_idx = state.haas_index(info.sample_index);
                let crack_spike = (comb_noise(crack_idx, crack_seed, 6) * 1.4
                                +  bright_noise(crack_idx, crack_seed) * 0.6)
                                * crack_spike_env;
                let crack_body = comb_noise(crack_idx,
                                             state.noise_seed(0xC4AC_FACE), 10) * crack_body_env;
                let crack = (crack_spike + crack_body * 0.5) * crack_amount * crack_vel * 1.6;

                // ─── Beater impact — bandpass filter excited by noise burst ───
                // The high-Q bandpass at 3.5 kHz rings briefly when excited;
                // we excite it with a sub-millisecond noise burst so the
                // ring carries the chaotic stick character. Much more like
                // a real beater than a bare bright-noise envelope.
                let beater_excite = if info.sample_index < 6 {
                    fast_noise(state.haas_index(info.sample_index),
                               state.noise_seed(0xBEAD_F00D)) * 1.2
                } else {
                    0.0
                };
                let beater_raw = state.snare_beater_bp.process(beater_excite);
                let beater = beater_raw * attack_amount * beater_vel * 4.0;

                // ─── Mix + broadband saturation + mid-band crack waveshaper ──
                let head_component = head * (1.0 - wire_mix_vel * 0.35);
                let wire_component = wires * (0.4 + wire_mix_vel * 0.9);
                let tone = head_component + wire_component + crack + beater;

                let decay_progress = 1.0 - top_amp1;
                let sat_eff = saturation_amount * sat_vel;
                let drive = 1.0 + sat_eff * (1.0 + decay_progress);
                let saturated = soft_saturate(tone, drive);

                // Mid-band crack: 2-5 kHz drive depth tied tightly to velocity.
                // At low velocity the mid-crack is nearly absent (ghost notes
                // stay round); at high velocity it's the dominant character.
                let mid = state.snare_crack_bp.process(saturated);
                let crack_drive = 1.5 + mid_crack_vel * 4.0;
                let mid_crushed = (mid * crack_drive).tanh() * 0.5;
                let saturated = saturated + mid_crushed * (0.30 + mid_crack_vel * 0.50);

                // Parallel comp — fast attack to catch the crack peak, medium
                // release so the wire/body sustains push through.
                let atk = 1.0 - (-1.0 / (info.sample_rate * 0.0005)).exp();   // 0.5 ms
                let rel = 1.0 - (-1.0 / (info.sample_rate * 0.060)).exp();    // 60 ms
                state.snare_comp_env = peak_follow(state.snare_comp_env, saturated, atk, rel);
                let comp_gain = compress_gain(state.snare_comp_env, 0.30, 4.5);
                let punchy = saturated * 0.55 + (saturated * comp_gain) * 0.90;

                tape_sat(punchy) * info.gain * SNARE_GAIN
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
                let ping_amount = params.as_ref()
                    .and_then(|p| p.ping_amount.map(r_to_f64))
                    .unwrap_or(0.18);
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
                let aggression = vel_curve(velocity);
                let brightness_vel = brightness * (0.55 + 0.85 * aggression * velocity_tilt);
                let attack_vel = 0.35 + 1.20 * aggression;
                let ping_vel = 0.15 + 1.30 * aggression;

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
                    let id = 0xCAFE_0000 ^ (i as u64).wrapping_mul(0x9E37);
                    let mode_freq = base_freq * ratio * state.freq_jitter(id);
                    if mode_freq >= info.sample_rate * 0.5 { continue; }
                    let mode_amp = (-t * decay_rate * decay_mult).exp();
                    let j = state.phase_jitter(id);
                    let phase = TAU * mode_freq * t + coupling_drive + j;
                    shimmer += phase.sin() * amp * mode_amp;
                }
                state.coupling_z = shimmer;

                // Highpass at 1.5 kHz keeps modes out of the kick band.
                let shimmer_hp = state.hihat_hp.process_hp(shimmer, info.sample_rate, 1500.0, 0.7);

                // ─── Air — band-decay cascade ─────────────────────────────────
                // Real cymbals shed high-frequency energy first: the >12 kHz
                // sizzle dies in 200-500ms while the 5-9 kHz content holds
                // for a full second-plus. Split the noise across two bands
                // with DIFFERENT decay rates so the cymbal "darkens" as it
                // rings, exactly like a real hi-hat.
                let white = fast_noise(state.haas_index(info.sample_index),
                                        state.noise_seed(0xCAFE_A11));
                let air_lo = state.hihat_bp_low.process(white);
                let air_hi = state.hihat_bp_high.process(white);
                let air_lo_env = (-t * decay_rate * 0.85).exp();   // 6 kHz lingers
                let air_hi_env = (-t * decay_rate * 1.55).exp();   // 11 kHz dies faster
                let air_signal = air_lo * air_lo_env * 1.6
                               + air_hi * air_hi_env * 1.2;

                // ─── Attack ping — pitched stick-contact bell tone ────────────
                // Closed hi-hats have a brief tonal ping at the attack (the
                // stick striking the edge of the cymbal). A 2.5 kHz sine
                // burst with a ~3ms decay; scaled hard by velocity so soft
                // hits stay airy and hard hits have a clearly audible "ting."
                let ping_freq = (info_freq * tune * 4.5).clamp(1800.0, 3500.0);
                let ping_env = (-t / 0.0030).exp();
                let ping_jit = state.phase_jitter(0x9_1B_F00D);
                let ping = (TAU * ping_freq * t + ping_jit).sin() * ping_env * ping_vel * ping_amount;

                // ─── Sharp attack — stick contact noise ───────────────────────
                let attack_env = (-t / 0.0015).exp();
                let attack_noise = bright_noise(state.haas_index(info.sample_index),
                                                 state.noise_seed(0x1234_56_F00D))
                                 * attack_env * attack_amount * attack_vel;

                let tone = shimmer_hp * 0.45 + air_signal * 0.55 + attack_noise + ping;

                tape_sat(tone) * info.gain * HIHAT_GAIN
            }
        }
    }
}
