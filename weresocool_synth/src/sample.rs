use crate::tables::{fast_exp, fast_sin, fast_tanh, svf_tan};
use crate::presets::{
    resolve_clap, resolve_cowbell, resolve_crash, resolve_hihat, resolve_kick, resolve_ride,
    resolve_rimshot, resolve_shaker, resolve_snare, resolve_tom, ResolvedClap, ResolvedCowbell,
    ResolvedCrash, ResolvedHiHat, ResolvedKick, ResolvedRide, ResolvedRimshot, ResolvedShaker,
    ResolvedSnare, ResolvedTom,
};
use crate::voice::{SampleInfo, Voice};
use std::f64::consts::PI;
use weresocool_ast::OscType;
use weresocool_shared::r_to_f64;

const TAU: f64 = PI * 2.0;

// Per-drum loudness compensation, calibrated so the default Kick, Snare,
// and HiHat land within 1 dB LUFS of each other at Gm 1 (enforced by
// `verify_drum_balance` in src/testing/loudness_balance_test.rs — rerun
// `measure_drum_loudness -- --nocapture` for fresh correction factors after
// any synthesis change). The kick is the anchor; the others match it.
// These scale only the output stage, so rebalancing never changes timbre.
const KICK_GAIN: f64 = 0.4;
const SNARE_GAIN: f64 = 0.67;
const HIHAT_GAIN: f64 = 1.18;
const CLAP_GAIN: f64 = 1.07;
const RIMSHOT_GAIN: f64 = 1.39;
const TOM_GAIN: f64 = 0.403;
const RIDE_GAIN: f64 = 1.377;
const CRASH_GAIN: f64 = 0.667;
const SHAKER_GAIN: f64 = 1.238;
const COWBELL_GAIN: f64 = 0.407;

// Default tuning when info.frequency falls outside the musical range we trust
// (e.g. silence ops setting freq to 0). Lets `Kick` still sound like a kick
// even when no header frequency is set.
const KICK_DEFAULT_FREQ: f64 = 60.0;
const SNARE_DEFAULT_FREQ: f64 = 185.0;
const HIHAT_DEFAULT_FREQ: f64 = 320.0;
// Clap/Rimshot are "unpitched" but still track Fm relative to this
// neutral frequency, so `rs | Fm 1/2` darkens the rings and `| Fm 9/8`
// section warps move them with the rest of the kit.
const CLAP_DEFAULT_FREQ: f64 = 165.0;
const RIMSHOT_DEFAULT_FREQ: f64 = 165.0;

/// The canonical drum-header frequency (`{ f: 60, ... }` — what every kit
/// in `kits/` uses). Tom, Ride, Crash, Shaker, and Cowbell are anchored to
/// their own register rather than reading `info.frequency` raw:
///
///   `f_base = base_pitch · tune · (note / 60)^pitch_track`
///
/// so `base_pitch` reads directly as "what Hz this drum is at a standard
/// header," and `pitch_track` decides how much the note moves it. Toms
/// track fully (1.0 — they are pitched instruments, and `Tom | Fm 2` is an
/// honest octave); cymbals and shakers track weakly (0.25-0.3), so `Fm`
/// colours the metal without transposing an unpitched sound off the map.
const DRUM_REF_FREQ: f64 = 60.0;

/// Number of plate modes in `presets::HIHAT_MODES` — shared by the hi-hat,
/// ride, and crash, and the width of the per-note mode cache in `DrumState`.
/// Waveguide loop gain: the feedback that leaves ~5% of the energy after
/// `decay_seconds`, for a loop `delay_samps` long. `0.05^x` written as
/// `exp(x·ln 0.05)` — the delay moves with the pitch envelope, so this is a
/// per-sample call on every kick, snare, and tom, and `exp` is several times
/// cheaper than the general `powf`.
#[inline]
fn ks_loop_gain(delay_samps: f64, decay_seconds: f64, sample_rate: f64) -> f64 {
    const LN_005: f64 = -2.995_732_273_553_991;
    fast_exp(LN_005 * delay_samps / (decay_seconds * sample_rate))
}

const HIHAT_MODE_COUNT: usize = 22;
/// Owners of the `DrumState` mode cache. 0 means "empty".
const MODE_KIND_HIHAT: u8 = 1;
const MODE_KIND_RIDE: u8 = 2;
const MODE_KIND_CRASH: u8 = 3;

/// Resolve a drum's register: anchor Hz, tuned, tracking the played note.
#[inline]
fn drum_register(base_pitch: f64, tune: f64, note_freq: f64, pitch_track: f64) -> f64 {
    base_pitch * tune * (note_freq / DRUM_REF_FREQ).powf(pitch_track)
}

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
        let g = svf_tan(cutoff.clamp(15.0, sr * 0.49) / sr);
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
        let g = svf_tan(cutoff.clamp(15.0, sr * 0.49) / sr);
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
        let g = svf_tan(cutoff.clamp(15.0, sr * 0.49) / sr);
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
    let shaped = fast_tanh(biased);
    let dc = fast_tanh(asymmetry);
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
    lp_z: f64,           // one-pole lowpass state for the feedback filter
    pending_excite: f64, // excitation accumulated since the last process()
}

impl Default for KarplusStrong {
    fn default() -> Self {
        // Sized for fundamentals as low as 20 Hz at 96 kHz (4800 samples).
        // Cheap memory; means we never have to resize at runtime.
        Self { buffer: vec![0.0; 5000], write_pos: 0, lp_z: 0.0, pending_excite: 0.0 }
    }
}

impl KarplusStrong {
    pub fn reset_state(&mut self) {
        for s in self.buffer.iter_mut() { *s = 0.0; }
        self.write_pos = 0;
        self.lp_z = 0.0;
        self.pending_excite = 0.0;
    }

    /// Inject an excitation sample. Accumulated into the NEXT `process()`
    /// write — excitation adds energy to the loop without touching the
    /// circulating content.
    #[inline]
    pub fn excite(&mut self, x: f64) {
        self.pending_excite += x;
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
        let feedback = fast_tanh(self.lp_z * loop_gain) * 0.95;

        // REPLACE the slot content (feedback + any new excitation). This
        // write must not accumulate onto the stale value from one buffer
        // revolution ago: `+=` here turned the loop into a self-sustaining
        // oscillator — energy accumulated until tanh saturation balanced
        // it, producing a tonal drone that SWELLED back ~200 ms after the
        // hit and sustained until the outer gain fade. (The kick masked
        // this because its KS output is shaped by the body envelope; the
        // snare's isn't, which is why snares sounded haunted.)
        self.buffer[self.write_pos % len] = feedback + self.pending_excite;
        self.pending_excite = 0.0;
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

/// Micro onset ramp — direct-noise components (snare crack, hat attack
/// noise, clap bursts, rim click) used to start at FULL amplitude on
/// sample zero: a one-sample step of up to ~0.2, i.e. a literal click
/// stapled onto every hit — and since drum noise is deliberately L/R
/// decorrelated, the click differed per channel ("pops on either side").
/// 0.35 ms is far below transient-perception time, so the snap survives;
/// only the step disappears.
#[inline]
fn onset_ramp(t: f64) -> f64 {
    (t / 0.00035).min(1.0)
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
    // Broadband wire BED — highpassed bright noise under the four resonant
    // bands. The bands alone are narrow peaks with nothing between them,
    // which reads as hollow metallic whistle; real wires are a DENSE
    // broadband rattle with resonant coloring on top.
    pub snare_wire_hp: Biquad,
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
    // Clap bandpasses on the burst-train noise.
    pub clap_bp1: Biquad,
    pub clap_bp2: Biquad,
    // Rimshot resonators — high-Q bandpasses rung by an impulse.
    pub rim_bp1: Biquad,
    pub rim_bp2: Biquad,
    // TOM — same architecture as the kick (waveguide + swept body lowpass +
    // shell ring + filter-excited click), retuned for a musical pitch bend
    // and a long open decay. Its own waveguide rather than the kick's: a
    // still-ringing kick tail can coexist with a tom note on one voice, and
    // sharing the delay line would splice one drum's energy into the other.
    pub tom_body: TptSvf,
    pub tom_ks: KarplusStrong,
    pub tom_click_bp: Biquad,
    pub tom_shell_bp: Biquad,
    pub tom_comp_env: f64,
    // RIDE — the plate bank's wash bands, a highpass, and the stick-ping
    // resonator. Separate from the crash's set for the same tail-overlap
    // reason as the tom.
    pub ride_bp_low: Biquad,
    pub ride_bp_high: Biquad,
    pub ride_hp: TptSvf,
    pub ride_ping_bp: Biquad,
    pub ride_coupling_z: f64,
    // CRASH — wash bands, highpass, plus the whole-cymbal lowpass whose
    // cutoff falls through the decay (the tail darkening).
    pub crash_bp_low: Biquad,
    pub crash_bp_high: Biquad,
    pub crash_hp: TptSvf,
    pub crash_lp: TptSvf,
    pub crash_coupling_z: f64,
    // SHAKER — two particle bands, the shell resonance under them, and the
    // pair of high-Q jingle rings that turn the family into a tambourine.
    pub shaker_bp_low: Biquad,
    pub shaker_bp_high: Biquad,
    pub shaker_shell_bp: Biquad,
    pub shaker_jingle_1: Biquad,
    pub shaker_jingle_2: Biquad,
    // COWBELL — the hollow "cup" bandpass and the mallet click resonator.
    pub cowbell_bp: Biquad,
    pub cowbell_click_bp: Biquad,
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
    // PERSISTENT NOTE CLOCK — samples since the current drum note's onset.
    // Drum envelopes are functions of time-since-note-on; ops are not. When
    // a drum rings into a following silence op (Seq [bd, Fm 0, ...]), the
    // op-relative sample index restarts at 0, which used to replay the
    // entire attack inside the "silence." This clock only resets on a true
    // note-on, so the tail decays continuously through silence ops.
    pub note_sample: usize,
    // PER-NOTE RESOLVED PARAMS — preset + user params flattened to f64 once
    // per note (None = not yet resolved this note; cleared by `reset()`).
    // Separate per-drum fields because an old drum's tail can coexist with
    // a new note of a different drum type on one DrumState. Also a perf
    // win: the params used to be re-read (with Rational64→f64 conversion)
    // every sample.
    pub resolved_kick: Option<ResolvedKick>,
    pub resolved_snare: Option<ResolvedSnare>,
    pub resolved_hihat: Option<ResolvedHiHat>,
    pub resolved_clap: Option<ResolvedClap>,
    pub resolved_rimshot: Option<ResolvedRimshot>,
    pub resolved_tom: Option<ResolvedTom>,
    pub resolved_ride: Option<ResolvedRide>,
    pub resolved_crash: Option<ResolvedCrash>,
    pub resolved_shaker: Option<ResolvedShaker>,
    pub resolved_cowbell: Option<ResolvedCowbell>,
    // SOFT CHOKE — when a note-on cuts a still-ringing drum, the old
    // output used to vanish in ONE sample (a 0.08-amplitude step at a
    // kick boundary = an audible broadband pop). Real drum machines
    // retrigger through analog envelopes and never step. At reset() the
    // last rendered output is carried into `choke_z`, which the voice
    // adds to the new note while decaying it over ~3 ms — the cut becomes
    // a fast fade under the new transient.
    pub choke_z: f64,
    pub last_out: f64,
    // PER-NOTE MODE CACHE — the plate-mode loop (hi-hat, ride, crash) reads
    // 22 modes every sample. Everything about a mode except its envelope
    // level is fixed for the whole note: the frequency multiplier, the phase
    // jitter, the amplitude tilt. Recomputing them per sample cost 22 `exp`
    // and 44 hash calls per sample per voice, which made a six-voice hat
    // pattern roughly four times the cost of an eight-voice kick pattern.
    //
    // `mode_kind` tags which family owns the cache (0 = empty, 1 = hi-hat,
    // 2 = ride, 3 = crash); a different family reseeds it. Only one family
    // sounds per voice at a time — drum→drum crossfade is suppressed in
    // `Voice::update`, so two drum voices never share one DrumState mid-note.
    pub mode_kind: u8,
    /// `ratio · mode_freq_scale · freq_jitter` — multiply by the (moving)
    /// base frequency to get the mode's frequency.
    pub mode_k: [f64; 22],
    /// Per-mode phase jitter, radians.
    pub mode_j: [f64; 22],
    /// `exp(-t · decay_rate · decay_mult)` at `mode_env_sample`.
    pub mode_env: [f64; 22],
    /// Per-sample decay multiplier: `exp(-decay_rate · decay_mult / sr)`.
    pub mode_env_coef: [f64; 22],
    /// Note-relative sample index `mode_env` currently holds. `usize::MAX`
    /// means "not seeded." Advancing by one sample multiplies; any other
    /// step (a seek, a crossfade re-read of the same sample) recomputes or
    /// reuses exactly, so the cache never desynchronizes from `t`.
    pub mode_env_sample: usize,
    /// The note's per-mode amplitudes, kept so the mode loop can be trimmed
    /// as modes fall silent.
    pub mode_amp: [f64; 22],
    /// How many leading modes are still audible. The mode table is ordered
    /// by decay rate, so the top modes die first (mode 21 sheds energy 9.2×
    /// faster than the fundamental) and the loop shrinks as the hit rings
    /// out. Only ever decreases within a note.
    pub mode_active: usize,
    // PER-NOTE ENVELOPE CACHE — every drum voice is a stack of exponential
    // decays, `exp(-t · rate)`, whose rates are fixed once the note's params
    // resolve. Evaluating them with `exp` per sample cost the snare eleven
    // transcendental calls a sample; stepped multiplicatively they cost one
    // multiply each. Same ownership tag and same exact-resync rules as the
    // mode cache above.
    pub env_kind: u8,
    pub env_val: [f64; DRUM_ENV_COUNT],
    pub env_coef: [f64; DRUM_ENV_COUNT],
    pub env_sample: usize,
    /// Per-note phase and frequency jitter for the snare's four head modes,
    /// and its compressor coefficients — all constant for the note.
    pub snare_jit: [f64; 4],
    pub snare_fjit: [f64; 4],
    pub snare_comp_atk: f64,
    pub snare_comp_rel: f64,
}

/// Envelope slots per drum voice. The snare uses the most (nine).
const DRUM_ENV_COUNT: usize = 10;
/// Owners of the `DrumState` envelope cache. 0 means "empty".
/// Envelope level under which a drum voice is treated as finished. −120 dB
/// relative to the note's own peak.
const DRUM_SILENCE: f64 = 1e-6;
const ENV_KIND_SNARE: u8 = 1;
const ENV_KIND_HIHAT: u8 = 2;

/// A mode is dropped once its remaining amplitude falls below this. At a
/// -180 dBFS contribution it is 60 dB under the noise floor of 24-bit audio
/// and some 200 dB under the drum's own peak.
const MODE_SILENCE: f64 = 1e-9;

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
        self.snare_wire_hp.reset_state();
        self.snare_shell_lp.reset_state();
        self.snare_ks.reset_state();
        self.snare_beater_bp.reset_state();
        self.snare_crack_bp.reset_state();
        self.snare_comp_env = 0.0;
        self.hihat_bp_low.reset_state();
        self.hihat_bp_high.reset_state();
        self.hihat_hp.reset_state();
        self.clap_bp1.reset_state();
        self.clap_bp2.reset_state();
        self.rim_bp1.reset_state();
        self.rim_bp2.reset_state();
        self.tom_body.reset_state();
        self.tom_ks.reset_state();
        self.tom_click_bp.reset_state();
        self.tom_shell_bp.reset_state();
        self.tom_comp_env = 0.0;
        self.ride_bp_low.reset_state();
        self.ride_bp_high.reset_state();
        self.ride_hp.reset_state();
        self.ride_ping_bp.reset_state();
        self.ride_coupling_z = 0.0;
        self.crash_bp_low.reset_state();
        self.crash_bp_high.reset_state();
        self.crash_hp.reset_state();
        self.crash_lp.reset_state();
        self.crash_coupling_z = 0.0;
        self.shaker_bp_low.reset_state();
        self.shaker_bp_high.reset_state();
        self.shaker_shell_bp.reset_state();
        self.shaker_jingle_1.reset_state();
        self.shaker_jingle_2.reset_state();
        self.cowbell_bp.reset_state();
        self.cowbell_click_bp.reset_state();
        self.coupling_z = 0.0;
        self.note_counter = self.note_counter.wrapping_add(1);
        self.note_sample = 0;
        self.resolved_kick = None;
        self.resolved_snare = None;
        self.resolved_hihat = None;
        self.resolved_clap = None;
        self.resolved_rimshot = None;
        self.resolved_tom = None;
        self.resolved_ride = None;
        self.resolved_crash = None;
        self.resolved_shaker = None;
        self.resolved_cowbell = None;
        // Seed the choke tail with the final output of the note being cut
        // (last_out already folds in any previous, still-decaying choke).
        self.choke_z = self.last_out;
        self.last_out = 0.0;
        self.mode_kind = 0;
        self.mode_env_sample = usize::MAX;
        self.env_kind = 0;
        self.env_sample = usize::MAX;
    }

    /// Bring the exponential-envelope cache in sync with sample `idx` of this
    /// note. `rates` are the per-note decay rates, in `exp(-t · rate)` form.
    /// Advancing one sample is one multiply per envelope; a re-read of the
    /// same sample, or any jump, resolves exactly — same contract as
    /// `sync_modes`.
    #[inline]
    pub fn sync_envs(&mut self, kind: u8, rates: &[f64], sample_rate: f64, idx: usize) {
        debug_assert!(rates.len() <= DRUM_ENV_COUNT);
        if self.env_kind == kind && self.env_sample != usize::MAX {
            if idx == self.env_sample {
                return;
            }
            if idx == self.env_sample + 1 {
                for i in 0..rates.len() {
                    self.env_val[i] *= self.env_coef[i];
                }
                self.env_sample = idx;
                return;
            }
        }

        let t = idx as f64 / sample_rate;
        for (i, rate) in rates.iter().enumerate() {
            self.env_val[i] = (-t * rate).exp();
            self.env_coef[i] = (-rate / sample_rate).exp();
        }
        self.env_kind = kind;
        self.env_sample = idx;
    }

    /// Bring the plate-mode cache in sync with sample `idx` of this note.
    ///
    /// Seeds it if a different family (or a new note) owns it, advances the
    /// envelopes by one multiply on the common case of a forward sample, and
    /// recomputes them exactly on any other jump. Re-reading the same sample
    /// — which happens when a crossfade renders the old and new oscillator
    /// against one shared DrumState — leaves the envelopes untouched, so the
    /// second read sees the same values as the first.
    #[inline]
    pub fn sync_modes(
        &mut self,
        kind: u8,
        id_base: u64,
        mode_freq_scale: f64,
        decay_rate: f64,
        sample_rate: f64,
        idx: usize,
        amps: &[f64; HIHAT_MODE_COUNT],
    ) {
        if self.mode_kind == kind && self.mode_env_sample != usize::MAX {
            if idx == self.mode_env_sample {
                return;
            }
            if idx == self.mode_env_sample + 1 {
                for i in 0..self.mode_active {
                    self.mode_env[i] *= self.mode_env_coef[i];
                }
                self.mode_env_sample = idx;
                self.trim_modes();
                return;
            }
        }

        let t = idx as f64 / sample_rate;
        for (i, (ratio, _, decay_mult)) in crate::presets::HIHAT_MODES.iter().copied().enumerate() {
            let id = id_base ^ (i as u64).wrapping_mul(0x9E37);
            self.mode_k[i] = ratio * mode_freq_scale * self.freq_jitter(id);
            self.mode_j[i] = self.phase_jitter(id);
            let d = decay_rate * decay_mult;
            self.mode_env[i] = (-t * d).exp();
            self.mode_env_coef[i] = (-d / sample_rate).exp();
            self.mode_amp[i] = amps[i];
        }
        self.mode_kind = kind;
        self.mode_env_sample = idx;
        self.mode_active = HIHAT_MODE_COUNT;
        self.trim_modes();
    }

    /// Drop trailing modes that have decayed below audibility.
    #[inline]
    fn trim_modes(&mut self) {
        while self.mode_active > 0
            && self.mode_env[self.mode_active - 1] * self.mode_amp[self.mode_active - 1].abs()
                < MODE_SILENCE
        {
            self.mode_active -= 1;
        }
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

                // ─── Per-note resolution + filter setup ───────────────────────
                // Preset + user params flatten to f64 ONCE per note (cache
                // invalidated by `DrumState::reset()` on note-on; persists
                // through tails carried into silence ops, exactly like the
                // filter coefficients set here).
                let info_freq = if info.frequency > 20.0 { info.frequency } else { KICK_DEFAULT_FREQ };
                if state.resolved_kick.is_none() {
                    let rk = resolve_kick(params.as_ref());
                    state.kick_click_bp.bandpass(info.sample_rate, rk.click_freq, 6.0);
                    let shell_hz = (info_freq * rk.tune * 1.78 + 18.0).clamp(60.0, 220.0);
                    state.kick_shell_bp.bandpass(info.sample_rate, shell_hz, 5.0);
                    state.resolved_kick = Some(rk);
                }
                let rk = state.resolved_kick.unwrap();
                let pre = rk.internal;
                let f_base = info_freq * rk.tune;
                let (pitch_decay, pitch_range, amp_decay) = (rk.pitch_decay, rk.pitch_range, rk.amp_decay);
                let (click_amount, saturation_amount) = (rk.click_amount, rk.saturation);
                let (hump_amount, shell_amount, ks_mix) = (rk.hump, rk.shell, rk.ks_mix);

                let velocity = info.gain.clamp(0.0, 1.0);
                // EXPRESSIVE VELOCITY: aggressive curve scales click, saturation,
                // and the body-LFO depth. Soft hits = rounded, no click, lots of
                // breath. Hard hits = saturated, click-heavy, punchy with no LFO.
                let aggression = vel_curve(velocity);
                let click_amount_vel = click_amount * (0.25 + 1.05 * aggression * rk.velocity_tilt);
                let sat_vel = 0.45 + 0.7 * aggression;       // 0.45-1.15× saturation
                let lfo_vel = 1.0 - 0.85 * aggression;       // soft hits breathe, hard hits don't
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

                // ─── Pitch-envelope phase integration ─────────────────────────
                // The SHAPE is a preset-internal knob: 808/wsc use the
                // two-stage knee (fast drop + slow settle — a pure single
                // exponential sounds too smooth there), 909 uses a single
                // exponential for its punchier straight drop. Both come with
                // closed-form integrals so the phase stays continuous.
                let (exp_pd, pd_integral) = pre.pitch_env.eval(t, pitch_decay);
                let env_factor = (pitch_range - 1.0) * pd_integral;
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
                let loop_gain = ks_loop_gain(delay_samps, amp_decay, info.sample_rate)
                                  .clamp(0.80, 0.998);
                // LP coef: more damping as the body decays → settling sound.
                // Use the amp-decay envelope hoisted up from below.
                let body_decay_env = fast_exp(-t * 3.0 / amp_decay);
                let lp_coef = 0.30 + 0.25 * (1.0 - body_decay_env);

                // Additive sine still contributes — gives us a tunable,
                // predictable fundamental. KS layer adds physical-model
                // character on top. Blend is a preset knob: 808 is sine-
                // dominant (predictable sub), acoustic is KS-dominant.
                let fm_mod = fast_sin(kick_phase * 2.0) * pre.fm_depth * exp_pd;
                let sine_fundamental = fast_sin(kick_phase + fm_mod);
                let ks_voice = state.kick_ks.process(delay_samps, loop_gain, lp_coef);
                let fundamental = sine_fundamental * pre.sine_level + ks_voice * ks_mix;

                // ─── Amplitude envelope with initial punch hump + body LFO ────
                // body_decay_env was computed above for the KS lp_coef.
                let tau_h = 0.004;
                let hump = hump_amount * (t / tau_h) * fast_exp(-t / tau_h) * std::f64::consts::E;
                let lfo_phase = TAU * 7.0 * t + state.phase_jitter(0x1CC0_AAAA);
                let body_lfo = 1.0 + 0.04 * lfo_vel * fast_sin(lfo_phase) * (1.0 - body_decay_env);
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
                let click = click_raw * click_amount_vel * pre.click_gain;

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

                // Cutoff sweeps from the preset's hi multiple of the
                // fundamental during the transient down to its lo multiple
                // as the body decays — 909 opens wide for the click, dust
                // barely opens at all.
                let pitch_env = 1.0 + (pitch_range - 1.0) * exp_pd;        // (R..1)
                let cutoff_hi = f_base * pre.cutoff_mult.0 * pitch_env;
                let cutoff_lo = f_base * pre.cutoff_mult.1;
                let cutoff = cutoff_lo + (cutoff_hi - cutoff_lo) * body_decay_env;
                let body_shaped = state.kick_body.process_lp(body_sat, info.sample_rate, cutoff, pre.body_q);

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
                let shell_env = fast_exp(-t * 3.0 / (amp_decay * 0.7));
                let shell = shell_ring * shell_env * shell_amount;

                // ─── Parallel compression — lifts body, keeps transient ───────
                // Fast attack so we catch the kick's initial spike; medium
                // release so the body holds its energy through the decay.
                // Then mix the compressed signal parallel with the dry: the
                // dry preserves the transient shape, the compressed lifts the
                // sustain. This is the classic "drum bus" compression sound.
                let dry = body_shaped + click + shell;
                let atk = 1.0 - fast_exp(-1.0 / (info.sample_rate * pre.comp.attack_s));
                let rel = 1.0 - fast_exp(-1.0 / (info.sample_rate * pre.comp.release_s));
                state.kick_comp_env = peak_follow(state.kick_comp_env, dry, atk, rel);
                let comp_gain = compress_gain(state.kick_comp_env, pre.comp.threshold, pre.comp.ratio);
                let punchy = dry * pre.comp.dry + (dry * comp_gain) * pre.comp.wet;

                // Output stage: tape-style asymmetric soft clip for "glue."
                // `drive_out` pushes harder into the clip — the trap-kick
                // speaker-knock move.
                tape_sat(punchy * pre.drive_out) * info.gain * KICK_GAIN * pre.gain_trim
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

                // ─── Per-note resolution + filter setup ───────────────────────
                // Preset + user params flatten ONCE per note. The wire
                // spectrum is modelled with FOUR bandpasses at the preset's
                // wire resonance frequencies (real wires aren't a smooth
                // hiss — they have specific resonances; wsc uses measured
                // 5.8/9.7/13.3/16.2 kHz, dust shifts the whole set down).
                // Q and amplitude taper with frequency, matching measured
                // snare spectra.
                let info_freq = if info.frequency > 20.0 { info.frequency } else { SNARE_DEFAULT_FREQ };
                if state.resolved_snare.is_none() {
                    let mut rs = resolve_snare(params.as_ref());
                    let pre = rs.internal;
                    // The snare's fundamental sits at its own register
                    // (`base_pitch`), tracking the played note only as far as
                    // `pitch_track` allows — so a low bass root doesn't drag
                    // the snare into the mud. wsc (185 Hz, track 1.0) keeps
                    // the exact historical `f_base = note · tune`.
                    rs.base_freq = pre.base_pitch * rs.tune
                        * (info_freq / SNARE_DEFAULT_FREQ).powf(pre.pitch_track);
                    state.snare_wires.bandpass(info.sample_rate, pre.wire_freqs[0] * rs.bright_shift, pre.wire_qs[0]);
                    state.snare_wires_2.bandpass(info.sample_rate, pre.wire_freqs[1] * rs.bright_shift, pre.wire_qs[1]);
                    state.snare_wires_3.bandpass(info.sample_rate, pre.wire_freqs[2] * rs.bright_shift, pre.wire_qs[2]);
                    state.snare_wires_4.bandpass(info.sample_rate, pre.wire_freqs[3] * rs.bright_shift, pre.wire_qs[3]);
                    state.snare_wire_hp.highpass(info.sample_rate, 2800.0 * rs.bright_shift, 0.7);
                    state.snare_beater_bp.bandpass(info.sample_rate, pre.beater_bp.0, pre.beater_bp.1);
                    state.snare_crack_bp.bandpass(info.sample_rate, rs.crack_freq, 1.4);
                    // Head-mode jitter and the compressor's time constants are
                    // fixed for the note — resolved here rather than re-derived
                    // (eight hashes and two `exp`) on every sample.
                    for i in 0..4 {
                        let id = 0x5_DEAD_0001 + i as u64;
                        state.snare_jit[i] = state.phase_jitter(id);
                        state.snare_fjit[i] = state.freq_jitter(id);
                    }
                    state.snare_comp_atk = 1.0 - fast_exp(-1.0 / (info.sample_rate * pre.comp.attack_s));
                    state.snare_comp_rel = 1.0 - fast_exp(-1.0 / (info.sample_rate * pre.comp.release_s));
                    state.resolved_snare = Some(rs);
                }
                let rs = state.resolved_snare.unwrap();
                let pre = rs.internal;
                let f_base = rs.base_freq;
                let (shell_decay, wire_decay, wire_mix) = (rs.shell_decay, rs.wire_decay, rs.wire_mix);
                let (shell_tune, attack_amount) = (rs.shell_tune, rs.attack_amount);
                let (shell_pitch_decay, shell_pitch_range) = (rs.shell_pitch_decay, rs.shell_pitch_range);
                let (head_damping_ratio, saturation_amount) = (rs.head_damping_ratio, rs.saturation);
                let (crack_amount, snare_ks_mix) = (rs.crack, rs.ks_mix);

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let wire_mix_vel = wire_mix * (0.55 + 0.85 * aggression * rs.velocity_tilt);
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
                // Every exponential this voice needs, stepped in one pass.
                // Order is the cache's contract — keep it in sync with the
                // reads below.
                let env_rates = [
                    shell_pitch_decay,
                    3.0 / shell_decay,
                    3.0 * 1.4 / shell_decay,
                    3.0 * head_damping_ratio / shell_decay,
                    3.0 * head_damping_ratio * 1.6 / shell_decay,
                    3.0 / 0.006,
                    3.0 / wire_decay,
                    1.0 / 0.0010,
                    1.0 / 0.0050,
                ];
                state.sync_envs(ENV_KIND_SNARE, &env_rates, info.sample_rate, info.sample_index);
                let env = state.env_val;

                // Nothing left to render: every envelope that drives this
                // voice — head, wires, crack — is past audibility, so the
                // rest of the chain can only produce silence. A drum note
                // that rings out under a long op used to keep paying full
                // price for the remainder.
                if env[1] < DRUM_SILENCE && env[6] < DRUM_SILENCE && env[8] < DRUM_SILENCE {
                    return 0.0;
                }

                let pitch_mult = 1.0 + shell_pitch_range * env[0];
                let top_f1 = f_base * pitch_mult;
                let top_f2 = f_base * pre.mode_ratios.0 * pitch_mult;          // (1,1) mode
                let bot_f1 = f_base * shell_tune * pitch_mult;
                let bot_f2 = f_base * shell_tune * pre.mode_ratios.1 * pitch_mult;

                let top_amp1 = env[1];
                let top_amp2 = env[2];
                let bot_amp1 = env[3];
                let bot_amp2 = env[4];

                // Raw head — sine modes with phase + freq jitter for L/R width.
                let [j1, j2, j3, j4] = state.snare_jit;
                let [fj1, fj2, fj3, fj4] = state.snare_fjit;
                let head_sines = fast_sin(TAU * top_f1 * fj1 * t + j1) * pre.mode_amps[0] * top_amp1
                               + fast_sin(TAU * top_f2 * fj2 * t + j2) * pre.mode_amps[1] * top_amp2
                               + fast_sin(TAU * bot_f1 * fj3 * t + j3) * pre.mode_amps[2] * bot_amp1
                               + fast_sin(TAU * bot_f2 * fj4 * t + j4) * pre.mode_amps[3] * bot_amp2;

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
                let snare_lg = ks_loop_gain(snare_delay, shell_decay, info.sample_rate)
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
                // Dense broadband bed (bright noise → highpass) carries the
                // "rattle"; the four resonant bands color it. Bands alone
                // sounded like hollow whistle. Band sum scaled down to make
                // room for the bed at equal wire level.
                let bed_raw = bright_noise(state.haas_index(info.sample_index),
                                            state.noise_seed(0x717E_BED));
                let bed = state.snare_wire_hp.process(bed_raw) * pre.wire_bed_gain;
                let bands = state.snare_wires.process(white) * pre.wire_gains[0]
                          + state.snare_wires_2.process(white) * pre.wire_gains[1]
                          + state.snare_wires_3.process(white) * pre.wire_gains[2]
                          + state.snare_wires_4.process(white) * pre.wire_gains[3];
                let wires_filtered = bed + bands * 0.65;
                // Attack-weighted hard: the first ~6 ms of wire spray IS the
                // "snap" of a snare — the tail is sizzle, not snap. The
                // attack component scales with velocity aggression so hard
                // hits pop and ghosts stay smooth.
                let wire_attack = env[5];
                let wire_tail = env[6];
                let sympathy = 0.4 + 0.6 * bot_amp1;
                let wire_snap = 0.85 * (0.6 + 0.6 * aggression);
                let wire_env = (wire_attack * wire_snap + wire_tail * 0.30) * sympathy * onset_ramp(t);
                let wires = wires_filtered * wire_env;

                // ─── Crack: noise burst with per-note seed variation ──────────
                // Two stages: a sub-ms spike (comb-shaped around 4-6 kHz) and
                // a slightly slower body. The per-note noise seed means
                // consecutive snare hits have DIFFERENT noise patterns —
                // critical for breaking the "drum machine" perceptual tell.
                let crack_spike_env = env[7];
                let crack_body_env = env[8];
                // TRANSIENTS MONO, TAILS WIDE: short noise bursts with
                // independent L/R noise have RANDOM interaural correlation —
                // each hit localizes randomly hard-left or hard-right, which
                // reads as "pops jumping side to side" on rolls/stutters.
                // The crack/beater (the transient) use a voice-independent
                // seed; only the 3-sample Haas lag differentiates channels —
                // a stable, slightly-wide center. Wires/air keep their
                // decorrelated width in the enveloped sustain.
                let crack_seed = (state.note_counter as u64).wrapping_mul(0xC4AC_BEEF_0001);
                let crack_idx = state.haas_index(info.sample_index);
                let crack_spike = (comb_noise(crack_idx, crack_seed, pre.crack_taps.0) * 1.4
                                +  bright_noise(crack_idx, crack_seed) * 0.6)
                                * crack_spike_env;
                let crack_body = comb_noise(crack_idx,
                                             (state.note_counter as u64).wrapping_mul(0xC4AC_FACE_0001), pre.crack_taps.1) * crack_body_env;
                let crack = (crack_spike + crack_body * 0.5) * crack_amount * crack_vel * pre.crack_gain
                    * onset_ramp(t);

                // ─── Beater impact — bandpass filter excited by noise burst ───
                // The high-Q bandpass at 3.5 kHz rings briefly when excited;
                // we excite it with a sub-millisecond noise burst so the
                // ring carries the chaotic stick character. Much more like
                // a real beater than a bare bright-noise envelope.
                let beater_excite = if info.sample_index < 6 {
                    fast_noise(state.haas_index(info.sample_index),
                               (state.note_counter as u64).wrapping_mul(0xBEAD_F00D_0001)) * 1.2
                } else {
                    0.0
                };
                let beater_raw = state.snare_beater_bp.process(beater_excite);
                let beater = beater_raw * attack_amount * beater_vel * pre.beater_gain;

                // ─── Mix: saturate the BODY only, bursts stay clean ──────────
                // soft_saturate's x/(1+|x|) asymptote at 1.0 used to sit on
                // the WHOLE mix — the crack/beater transient slammed into the
                // ceiling while the body sat in the linear zone, collapsing
                // an ~8:1 transient-to-body ratio down to ~2:1. No amount of
                // burst gain could add snap; it only saturated harder (and
                // the `attack` macro audibly went BACKWARDS). Saturating the
                // shell alone keeps the warmth where it belongs and lets the
                // transient stand clean on top.
                let head_component = head * (1.0 - wire_mix_vel * 0.35);
                let wire_component = wires * (0.4 + wire_mix_vel * 0.9);

                let decay_progress = 1.0 - top_amp1;
                let sat_eff = saturation_amount * sat_vel;
                let drive = 1.0 + sat_eff * (1.0 + decay_progress);
                let body_sat = soft_saturate(head_component, drive);
                let tone = body_sat + wire_component + crack + beater;

                // Mid-band crack: 2-5 kHz drive depth tied tightly to velocity.
                // At low velocity the mid-crack is nearly absent (ghost notes
                // stay round); at high velocity it's the dominant character.
                let mid = state.snare_crack_bp.process(tone);
                let crack_drive = pre.mid_crack_drive_k.0 + mid_crack_vel * pre.mid_crack_drive_k.1;
                let mid_crushed = fast_tanh(mid * crack_drive) * 0.5;
                let saturated = tone + mid_crushed * (pre.mid_crack_mix_k.0 + mid_crack_vel * pre.mid_crack_mix_k.1);

                // Parallel comp — fast attack to catch the crack peak, medium
                // release so the wire/body sustains push through.
                state.snare_comp_env = peak_follow(
                    state.snare_comp_env, saturated, state.snare_comp_atk, state.snare_comp_rel);
                let comp_gain = compress_gain(state.snare_comp_env, pre.comp.threshold, pre.comp.ratio);
                let punchy = saturated * pre.comp.dry + (saturated * comp_gain) * pre.comp.wet;

                tape_sat(punchy * pre.drive_out) * info.gain * SNARE_GAIN * pre.gain_trim
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

                // ─── Per-note resolution + filter setup ───────────────────────
                // Bandpass cascade on noise = metallic "tssh." The mode
                // highpass uses TPT SVF; we don't sweep it (cymbals don't),
                // but the TPT version is more numerically robust for the
                // very high cutoff we run. Air band centers/Qs are preset
                // knobs (808 is darker, 909 airier).
                let info_freq = if info.frequency > 20.0 { info.frequency } else { HIHAT_DEFAULT_FREQ };
                if state.resolved_hihat.is_none() {
                    let rh = resolve_hihat(params.as_ref(), *open);
                    let pre = rh.internal;
                    state.hihat_bp_low.bandpass(info.sample_rate, pre.air_lo.0, pre.air_lo.1);
                    state.hihat_bp_high.bandpass(info.sample_rate, pre.air_hi.0, pre.air_hi.1);
                    state.coupling_z = 0.0;
                    state.resolved_hihat = Some(rh);
                }
                let rh = state.resolved_hihat.unwrap();
                let pre = rh.internal;
                let (tune, decay_rate, shimmer_mult) = (rh.tune, rh.decay_rate, rh.shimmer);
                let (brightness, attack_amount) = (rh.brightness, rh.attack_amount);
                let (ping_amount, pitch_drop) = (rh.ping_amount, rh.pitch_drop);

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let brightness_vel = brightness * (0.55 + 0.85 * aggression * rh.velocity_tilt);
                let attack_vel = 0.35 + 1.20 * aggression;
                let ping_vel = 0.15 + 1.30 * aggression;

                // Every exponential this voice needs, stepped in one pass —
                // the order here is the cache's contract, matching the reads
                // below.
                let env_rates = [
                    6.0,
                    decay_rate * pre.air_lo.3,
                    decay_rate * pre.air_hi.3,
                    1.0 / pre.ping_tau,
                    1.0 / 0.0015,
                ];
                state.sync_envs(ENV_KIND_HIHAT, &env_rates, info.sample_rate, info.sample_index);
                let env = state.env_val;

                // Slight pitch droop — cymbals lose high-end energy first.
                let pitch_drop_mult = 1.0 + pitch_drop * env[0];
                let base_freq = info_freq * tune * shimmer_mult * pitch_drop_mult;

                // 22-mode cymbal — first 22 Bessel-function zeros for a circular
                // plate (the actual physics of cymbal vibration), with three
                // intentional close pairs to create *beating*. Real cymbals have
                // 50-100 modes; 22 is plenty for perceptual realism while
                // keeping the compute reasonable. The table lives in
                // `presets::HIHAT_MODES`; the preset bends it via
                // `mode_freq_scale` (cymbal size) and the per-note tilted
                // amplitudes in `rh.mode_amp`. Brightness keeps scaling
                // modes 8+ per-sample (it's velocity-driven).
                //
                // Each mode also receives a tiny phase modulation from
                // `coupling_z` — the sum of the previous sample's mode outputs.
                // This is the nonlinear coupling that makes modes "talk to each
                // other" and creates the alive, shimmering character of real
                // metal instead of a static stacked-sines chord.
                let coupling_drive = state.coupling_z * pre.coupling;

                // Per-note constants (mode multipliers, phase jitter, decay
                // envelopes) come from the cache; per sample only the moving
                // base frequency and the coupling phase change. `w` folds
                // `TAU · base_freq · t` so each mode costs one multiply for
                // its frequency, one for its phase, and one `sin`.
                state.sync_modes(MODE_KIND_HIHAT, 0xCAFE_0000, pre.mode_freq_scale, decay_rate, info.sample_rate, info.sample_index, &rh.mode_amp);
                let w = TAU * base_freq * t;
                let nyquist = info.sample_rate * 0.5;
                let mut shimmer_low = 0.0;
                let mut shimmer_high = 0.0;
                for i in 0..state.mode_active {
                    let k = state.mode_k[i];
                    if base_freq * k >= nyquist { continue; }
                    let phase = w * k + coupling_drive + state.mode_j[i];
                    let voice = fast_sin(phase) * rh.mode_amp[i] * state.mode_env[i];
                    if i >= 8 { shimmer_high += voice } else { shimmer_low += voice }
                }
                // Brightness scales modes 8+ only — folded in once instead of
                // per mode.
                let shimmer = shimmer_low + shimmer_high * brightness_vel;
                state.coupling_z = shimmer;

                // Highpass keeps modes out of the kick band.
                let shimmer_hp = state.hihat_hp.process_hp(shimmer, info.sample_rate, pre.hp_cutoff, 0.7);

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
                let air_lo_env = env[1]; // low band lingers
                let air_hi_env = env[2]; // high band dies faster
                let air_signal = (air_lo * air_lo_env * pre.air_lo.2
                               + air_hi * air_hi_env * pre.air_hi.2)
                               * onset_ramp(t);

                // ─── Attack ping — pitched stick-contact bell tone ────────────
                // Closed hi-hats have a brief tonal ping at the attack (the
                // stick striking the edge of the cymbal). A 2.5 kHz sine
                // burst with a ~3ms decay; scaled hard by velocity so soft
                // hits stay airy and hard hits have a clearly audible "ting."
                let ping_freq = (info_freq * tune * pre.ping_freq_mult).clamp(pre.ping_clamp.0, pre.ping_clamp.1);
                let ping_env = env[3];
                let ping_jit = state.phase_jitter(0x9_1B_F00D);
                let ping = fast_sin(TAU * ping_freq * t + ping_jit) * ping_env * ping_vel * ping_amount;

                // ─── Sharp attack — stick contact noise ───────────────────────
                let attack_env = env[4];
                let attack_noise = bright_noise(state.haas_index(info.sample_index),
                                                 (state.note_counter as u64).wrapping_mul(0x1234_56_F00D))
                                 * attack_env * attack_amount * attack_vel
                                 * onset_ramp(t);

                let tone = shimmer_hp * pre.mix.0 + air_signal * pre.mix.1 + attack_noise + ping;

                tape_sat(tone) * info.gain * HIHAT_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // CLAP — burst train + resonant tail (the analog clap recipe)
            //
            // A real clap is several hands striking within ~30 ms. Analog
            // machines fake it with a retriggered fast noise envelope (the
            // burst train) into a resonant ~1 kHz bandpass, then hold a
            // longer noise tail ("the room"). The per-note spacing jitter
            // means no two claps are the same — the burst rhythm itself is
            // randomized, which is most of what makes a clap sound human.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Clap { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                if state.resolved_clap.is_none() {
                    let rc = resolve_clap(params.as_ref());
                    let pre = rc.internal;
                    let info_freq = if info.frequency > 20.0 { info.frequency } else { CLAP_DEFAULT_FREQ };
                    let pitch = info_freq / CLAP_DEFAULT_FREQ;
                    state.clap_bp1.bandpass(info.sample_rate, rc.bp1_freq * rc.tune * pitch, pre.bp1_q);
                    state.clap_bp2.bandpass(info.sample_rate, rc.bp1_freq * rc.tune * pitch * pre.bp2_ratio, pre.bp2_q);
                    state.resolved_clap = Some(rc);
                }
                let rc = state.resolved_clap.unwrap();
                let pre = rc.internal;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                // Velocity: hot claps spit (sharper bursts, brighter top).
                let burst_tau = rc.burst_tau * (1.15 - 0.45 * aggression * rc.velocity_tilt);
                let bp2_vel = rc.bp2_gain * (0.5 + 0.9 * aggression);

                // ─── Burst-train envelope ─────────────────────────────────────
                // n_bursts retriggered exponentials with per-note jittered
                // spacing, each successive burst a touch louder, then a
                // longer tail from the last burst.
                let n = pre.n_bursts.max(1);
                let mut env = 0.0;
                let mut burst_start = 0.0;
                let mut last_start = 0.0;
                let mut last_level = 1.0;
                for k in 0..n {
                    let jit = state.phase_jitter(0xC1A9_0000 ^ (k as u64))
                        / (std::f64::consts::PI / 15.0); // [-1, 1]
                    let spacing = rc.spacing * (1.0 + jit * pre.spacing_jitter);
                    let level = pre.burst_growth.powi(k as i32);
                    if t >= burst_start {
                        let local = t - burst_start;
                        // Each burst replaces the envelope (retrigger), the
                        // hallmark sawtooth-envelope shape of analog claps.
                        // Micro-ramped so the retrigger is not a step.
                        env = level * fast_exp(-local / burst_tau) * onset_ramp(local);
                        last_start = burst_start;
                        last_level = level;
                    }
                    burst_start += spacing;
                }
                // Tail takes over after the final burst's fast decay.
                let since_last = (t - last_start).max(0.0);
                let tail = last_level * pre.tail_level * fast_exp(-since_last / rc.tail_tau);
                let env = env.max(tail);

                // ─── Noise through the clap bands ────────────────────────────
                // Per-voice seeds + Haas: claps are naturally WIDE (many
                // hands, different positions) so we keep full decorrelation.
                let idx = state.haas_index(info.sample_index);
                // One machine voice — mono noise; Haas alone gives stable width
                // (the original 808 clap is a mono circuit).
                let white = fast_noise(idx, (state.note_counter as u64).wrapping_mul(0xC1A9_F00D));
                let body = state.clap_bp1.process(white);
                let top = state.clap_bp2.process(white);
                let tone = (body + top * bp2_vel) * env;

                // Saturation rounds the burst peaks into each other —
                // glues the train into one perceived "clap".
                let sat = soft_saturate(tone * (1.0 + rc.saturation * 2.0), 1.0)
                    * (1.0 + rc.saturation * 0.4);

                tape_sat(sat * pre.drive_out) * info.gain * CLAP_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // RIMSHOT — stick on rim+head: two inharmonic resonator rings
            // (the woody/metallic "tock") + a sharp click. Very short, dry.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Rimshot { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                if state.resolved_rimshot.is_none() {
                    let rr = resolve_rimshot(params.as_ref());
                    let pre = rr.internal;
                    let info_freq = if info.frequency > 20.0 { info.frequency } else { RIMSHOT_DEFAULT_FREQ };
                    let pitch = info_freq / RIMSHOT_DEFAULT_FREQ;
                    state.rim_bp1.bandpass(info.sample_rate, rr.ring_freq * rr.tune * pitch, pre.ring_q.0);
                    state.rim_bp2.bandpass(info.sample_rate, rr.ring_freq * rr.tune * pitch * pre.ring2_ratio, pre.ring_q.1);
                    state.resolved_rimshot = Some(rr);
                }
                let rr = state.resolved_rimshot.unwrap();
                let pre = rr.internal;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let click_vel = 0.35 + 1.2 * aggression * rr.velocity_tilt;

                // Impulse + brief noise chatter excites both resonators;
                // their Q does the ringing, an outer exponential bounds the
                // length (`length` macro).
                let excite = if info.sample_index < 3 {
                    1.0 - info.sample_index as f64 * 0.3
                        + fast_noise(info.sample_index, state.noise_seed(0x4131_BEEF)) * 0.4
                } else {
                    0.0
                };
                let ring_env = fast_exp(-t / rr.ring_tau);
                let ring1 = state.rim_bp1.process(excite) * pre.ring_mix.0;
                let ring2 = state.rim_bp2.process(excite) * pre.ring_mix.1 * rr.ring2_gain;
                // The TOCK must clearly carry — a click 10× the ring reads
                // as a bare pop, especially hard-panned.
                let rings = (ring1 + ring2) * ring_env * 34.0;

                // Stick click — 1 ms of bright noise.
                let click_env = fast_exp(-t / 0.0010);
                let click = bright_noise(state.haas_index(info.sample_index),
                                          (state.note_counter as u64).wrapping_mul(0x4131_F00D))
                    * click_env * rr.attack_amount * click_vel * 0.65
                    * onset_ramp(t);

                tape_sat(rings + click) * info.gain * RIMSHOT_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // TOM — a struck membrane with nothing to hide behind.
            //
            // Architecturally the kick's sibling: Karplus-Strong waveguide body,
            // sine modes on top, a lowpass that closes as the drum decays, a
            // wooden shell ring, and a filter-excited stick click. Three things
            // make it a tom rather than a high kick:
            //
            //   1. The pitch bend is MUSICAL (~a whole tone, `pitch_range` 1.22)
            //      instead of the kick's two-octave drop. A tom that swoops is a
            //      tom that sounds broken.
            //   2. The modes are the real circular-membrane ratios and they are
            //      LOUD — a kick buries them under the sub, a tom is mostly them.
            //   3. Higher modes decay faster than the fundamental (`mode_decays`),
            //      which is what makes the tail settle toward pure pitch the way
            //      a real head does.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Tom { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let note_freq = if info.frequency > 20.0 { info.frequency } else { DRUM_REF_FREQ };
                if state.resolved_tom.is_none() {
                    let mut rt = resolve_tom(params.as_ref());
                    let pre = rt.internal;
                    rt.base_freq =
                        drum_register(pre.base_pitch, rt.tune, note_freq, pre.pitch_track);
                    state.tom_click_bp.bandpass(info.sample_rate, pre.click_bp.0, pre.click_bp.1);
                    let shell_hz = (rt.base_freq * pre.shell_ratio).clamp(80.0, 5000.0);
                    state.tom_shell_bp.bandpass(info.sample_rate, shell_hz, pre.shell_q);
                    state.resolved_tom = Some(rt);
                }
                let rt = state.resolved_tom.unwrap();
                let pre = rt.internal;
                let f_base = rt.base_freq;
                let amp_decay = rt.amp_decay;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let click_vel = 0.25 + 1.05 * aggression * rt.velocity_tilt;
                let sat_vel = 0.45 + 0.75 * aggression;
                // Harder hits bend further — a real head stretches more under a
                // heavier stroke. This is most of why an accented tom reads as
                // "hit harder" rather than just "louder".
                let bend = 1.0 + (rt.pitch_range - 1.0) * (0.55 + 0.75 * aggression);

                // Waveguide excitation. Voice-independent seed: the tom body is
                // a low, centred sound, and decorrelating it across L/R would
                // smear the pitch rather than widen it.
                // `KarplusStrong::process` returns the buffer contents one full
                // delay period back, so the loop outputs literal silence until
                // the first circulation completes — 9 ms at 110 Hz. On a kick
                // that is hidden under a long envelope; on a tom it splits the
                // hit into an audible "tk … boom". Feeding the burst straight
                // through as well as into the loop closes the gap, and it is
                // physically right: the strike is heard at the head before the
                // wave has been anywhere.
                let mut ks_direct = 0.0;
                if t < 0.003 {
                    let mono_seed =
                        (state.note_counter as u64).wrapping_mul(0x70A2_5EED_1234_ABCD);
                    let burst = fast_noise(info.sample_index, mono_seed) * 0.55;
                    state.tom_ks.excite(burst);
                    ks_direct = burst;
                }

                // ─── Pitch bend ──────────────────────────────────────────────
                // Single exponential with its closed-form integral, so the mode
                // phases stay continuous while the pitch moves.
                let exp_pd = fast_exp(-t / rt.pitch_decay);
                let pd_integral = rt.pitch_decay * (1.0 - exp_pd);
                let env_factor = (bend - 1.0) * pd_integral;
                let pitch_now = 1.0 + (bend - 1.0) * exp_pd;

                // ─── Membrane modes ──────────────────────────────────────────
                let mut modes = 0.0;
                for i in 0..4 {
                    let id = 0x70_0000 ^ (i as u64).wrapping_mul(0x9E37);
                    let mode_f = f_base * pre.mode_ratios[i] * state.freq_jitter(id);
                    if mode_f >= info.sample_rate * 0.45 {
                        continue;
                    }
                    let phase = TAU * mode_f * (t + env_factor) + state.phase_jitter(id);
                    let amp = fast_exp(-t * 3.0 * pre.mode_decays[i] / amp_decay);
                    modes += fast_sin(phase) * pre.mode_amps[i] * amp;
                }

                // ─── Waveguide body ──────────────────────────────────────────
                let freq_now = f_base * pitch_now;
                let delay_samps = info.sample_rate / freq_now.max(20.0);
                let loop_gain = ks_loop_gain(delay_samps, amp_decay, info.sample_rate)
                    .clamp(0.80, 0.998);
                let body_decay_env = fast_exp(-t * 3.0 / amp_decay);
                let lp_coef = 0.30 + 0.28 * (1.0 - body_decay_env);
                let ks_voice =
                    state.tom_ks.process(delay_samps, loop_gain, lp_coef) + ks_direct;
                let fundamental = modes * pre.sine_level + ks_voice * rt.ks_mix;
                let body = fundamental * body_decay_env;

                // ─── Stick click ─────────────────────────────────────────────
                let click_excite = if info.sample_index < 4 {
                    let ramp = 1.0 - info.sample_index as f64 * 0.25;
                    let chaos = fast_noise(info.sample_index, state.noise_seed(0x70C1_1CC0));
                    ramp + chaos * 0.5
                } else if t < 0.001 {
                    fast_noise(info.sample_index, state.noise_seed(0x70C1_FACE)) * 0.3
                } else {
                    0.0
                };
                // Resonator ring for the pitch of the contact, plus a short
                // broadband burst for its texture. The ring alone lasts well
                // under a millisecond at any Q that isn't audibly tonal, which
                // is far too narrow to read as a stick against a 110 Hz body.
                // The noise half is the one part of the tom that is
                // stereo-decorrelated — body centred, contact wide.
                // Comb-shaped rather than `bright_noise`: differentiated noise
                // centres around 5-6 kHz, which is a rimshot, not wood on a
                // head. A tap of 8 puts the peak near 3 kHz.
                let stick = comb_noise(
                    state.haas_index(info.sample_index),
                    state.noise_seed(0x70_57_1C_C0),
                    8,
                ) * fast_exp(-t / 0.0013)
                    * pre.stick_gain
                    * onset_ramp(t);
                let click = (state.tom_click_bp.process(click_excite) * pre.click_gain + stick)
                    * rt.attack_amount
                    * click_vel;

                // ─── Saturation into a closing lowpass ───────────────────────
                let sat_eff = rt.saturation * sat_vel;
                let drive = 1.0 + sat_eff * 3.0 * body_decay_env;
                let asym = sat_eff * 0.35 * body_decay_env;
                let body_sat = asym_saturate(body, drive, asym) * (1.0 + sat_eff * 0.5);
                let cutoff_hi = f_base * pre.cutoff_mult.0 * rt.tone_scale * pitch_now;
                let cutoff_lo = f_base * pre.cutoff_mult.1 * rt.tone_scale;
                let cutoff = cutoff_lo + (cutoff_hi - cutoff_lo) * body_decay_env;
                let body_shaped =
                    state.tom_body.process_lp(body_sat, info.sample_rate, cutoff, pre.body_q);

                // ─── Wooden shell ────────────────────────────────────────────
                let shell_excite = if info.sample_index < 8 {
                    fast_noise(info.sample_index, state.noise_seed(0x70_5BE1_1F00))
                } else {
                    0.0
                };
                let shell = state.tom_shell_bp.process(shell_excite)
                    * fast_exp(-t * 3.0 / (amp_decay * 0.5))
                    * rt.shell;

                // ─── Parallel compression + output stage ─────────────────────
                let dry = body_shaped + click + shell;
                let atk = 1.0 - fast_exp(-1.0 / (info.sample_rate * pre.comp.attack_s));
                let rel = 1.0 - fast_exp(-1.0 / (info.sample_rate * pre.comp.release_s));
                state.tom_comp_env = peak_follow(state.tom_comp_env, dry, atk, rel);
                let comp_gain =
                    compress_gain(state.tom_comp_env, pre.comp.threshold, pre.comp.ratio);
                let punchy = dry * pre.comp.dry + (dry * comp_gain) * pre.comp.wet;

                tape_sat(punchy * pre.drive_out) * info.gain * TOM_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // RIDE — articulation first, wash second.
            //
            // A ride is identified by its PING: a clear, repeatable, pitched
            // stroke that survives inside a dense mix. The wash is what the
            // pattern accumulates into over a phrase, not what a single hit
            // sounds like. So the ping is synthesized explicitly (a decaying
            // sine at the strike frequency, plus a bandpass rung by a sub-
            // millisecond noise burst for the chaotic stick contact), the bell
            // partials sustain underneath it, and the noise wash BUILDS from
            // nothing over `wash_build` rather than starting at full level.
            //
            // The 22-mode Bessel plate bank is shared with the hi-hat, run at a
            // much lower `mode_freq_scale` (a bigger cymbal) and a far slower
            // decay, with heavier nonlinear coupling — large plates are more
            // nonlinear, which is where the shimmer and beating come from.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Ride { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let note_freq = if info.frequency > 20.0 { info.frequency } else { DRUM_REF_FREQ };
                if state.resolved_ride.is_none() {
                    let mut rr = resolve_ride(params.as_ref());
                    let pre = rr.internal;
                    rr.base_freq =
                        drum_register(pre.base_pitch, rr.tune, note_freq, pre.pitch_track);
                    let track = rr.base_freq / pre.base_pitch;
                    state.ride_bp_low.bandpass(info.sample_rate, pre.wash_lo.0 * track, pre.wash_lo.1);
                    state.ride_bp_high.bandpass(info.sample_rate, pre.wash_hi.0 * track, pre.wash_hi.1);
                    state.ride_ping_bp.bandpass(info.sample_rate, pre.ping_bp.0 * track, pre.ping_bp.1);
                    state.ride_coupling_z = 0.0;
                    state.resolved_ride = Some(rr);
                }
                let rr = state.resolved_ride.unwrap();
                let pre = rr.internal;
                let f_base = rr.base_freq * rr.shimmer;
                let decay_rate = rr.decay_rate;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let brightness_vel = rr.brightness * (0.55 + 0.85 * aggression * rr.velocity_tilt);
                // A ride's dynamic range lives almost entirely in the ping. Soft
                // strokes are nearly pure wash; hard strokes are all articulation.
                let ping_vel = 0.20 + 1.35 * aggression;
                let bell_vel = 0.45 + 0.90 * aggression;

                // ─── Plate modes ─────────────────────────────────────────────
                let coupling_drive = state.ride_coupling_z * pre.coupling;
                let nyquist = info.sample_rate * 0.5;
                state.sync_modes(MODE_KIND_RIDE, 0x21DE_0000, pre.mode_freq_scale, decay_rate, info.sample_rate, info.sample_index, &rr.mode_amp);
                let w = TAU * f_base * t;
                let mut shimmer_low = 0.0;
                let mut shimmer_high = 0.0;
                for i in 0..state.mode_active {
                    let k = state.mode_k[i];
                    let mode_freq = f_base * k;
                    // Fade modes out as they approach Nyquist instead of cutting
                    // them: `mode_freq` moves with `tune`/`Fm`, and a hard cull
                    // makes a mode appear or vanish on a step.
                    let rolloff = ((nyquist * 0.92 - mode_freq) / (nyquist * 0.08)).clamp(0.0, 1.0);
                    if rolloff <= 0.0 {
                        continue;
                    }
                    let phase = w * k + coupling_drive + state.mode_j[i];
                    let voice = fast_sin(phase) * rr.mode_amp[i] * state.mode_env[i] * rolloff;
                    if i >= 8 { shimmer_high += voice } else { shimmer_low += voice }
                }
                let shimmer = shimmer_low + shimmer_high * brightness_vel;
                state.ride_coupling_z = shimmer;

                // ─── Bell partials ───────────────────────────────────────────
                // Low, inharmonic, and much longer-lived than the plate modes.
                // This is the sustained tone you hear under a ride pattern — the
                // thing that makes repeated hits sound like one instrument
                // ringing rather than a series of separate noises.
                let mut bell = 0.0;
                for i in 0..4 {
                    let id = 0xBE11_0000 ^ (i as u64).wrapping_mul(0x9E37);
                    let bf = f_base * pre.bell_ratios[i] * state.freq_jitter(id);
                    if bf >= nyquist * 0.9 {
                        continue;
                    }
                    let env = fast_exp(-t * decay_rate * pre.bell_decay * (1.0 + i as f64 * 0.35));
                    bell += fast_sin(TAU * bf * t + state.phase_jitter(id)) * pre.bell_amps[i] * env;
                }
                bell *= rr.bell_amount * bell_vel;

                let plate = state.ride_hp.process_hp(
                    shimmer + bell,
                    info.sample_rate,
                    pre.hp_cutoff,
                    0.7,
                );

                // ─── Wash — two bands, building, then decaying ────────────────
                let white = fast_noise(
                    state.haas_index(info.sample_index),
                    state.noise_seed(0x21DE_A11),
                );
                let build = 1.0 - fast_exp(-t / pre.wash_build);
                let wash_lo = state.ride_bp_low.process(white)
                    * fast_exp(-t * decay_rate * pre.wash_lo.3)
                    * pre.wash_lo.2;
                let wash_hi = state.ride_bp_high.process(white)
                    * fast_exp(-t * decay_rate * pre.wash_hi.3)
                    * pre.wash_hi.2;
                let wash = (wash_lo + wash_hi) * build * rr.wash * onset_ramp(t);

                // ─── Stick ping ──────────────────────────────────────────────
                // Two parts. The sine carries the PITCH of the stroke (a filter
                // ring at a usable Q dies in about a millisecond — far too fast
                // to read as pitched), and the bandpass rung by a short noise
                // burst carries the chaotic contact grit on top of it.
                let ping_freq = (pre.ping_bp.0 * (rr.base_freq / pre.base_pitch))
                    .clamp(300.0, nyquist * 0.8);
                let ping_env = fast_exp(-t / pre.ping_tau);
                let ping_tone =
                    fast_sin(TAU * ping_freq * t + state.phase_jitter(0x21DE_9146)) * ping_env;
                let ping_excite = if info.sample_index < 6 {
                    fast_noise(
                        state.haas_index(info.sample_index),
                        (state.note_counter as u64).wrapping_mul(0x21DE_F00D_0001),
                    ) * 1.2
                } else {
                    0.0
                };
                let ping_grit = state.ride_ping_bp.process(ping_excite) * pre.ping_gain;
                let ping = (ping_tone * 0.75 + ping_grit * 0.55)
                    * rr.attack_amount
                    * ping_vel
                    * onset_ramp(t);

                let tone = plate * pre.mix.0 + wash * pre.mix.1 + ping;

                tape_sat(tone) * info.gain * RIDE_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // CRASH — the same plate, played to explode.
            //
            // The defining feature is the SWELL. A crash does not begin at full
            // level: the strike injects energy at one point and it spreads
            // across the plate over 5-20 ms, so the level RISES into a peak and
            // only then decays. Skip that and the result reads as "someone
            // turned up a hi-hat," no matter how long the tail is.
            //
            // The second feature is that the tail darkens fast. Per-mode decay
            // multipliers already shed the high modes first; on top of that the
            // whole cymbal runs through a lowpass whose cutoff falls from
            // `lp_sweep.0` to `lp_sweep.1` as it rings out.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Crash { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let note_freq = if info.frequency > 20.0 { info.frequency } else { DRUM_REF_FREQ };
                if state.resolved_crash.is_none() {
                    let mut rc = resolve_crash(params.as_ref());
                    let pre = rc.internal;
                    rc.base_freq =
                        drum_register(pre.base_pitch, rc.tune, note_freq, pre.pitch_track);
                    let track = rc.base_freq / pre.base_pitch;
                    state.crash_bp_low.bandpass(info.sample_rate, pre.wash_lo.0 * track, pre.wash_lo.1);
                    state.crash_bp_high.bandpass(info.sample_rate, pre.wash_hi.0 * track, pre.wash_hi.1);
                    state.crash_coupling_z = 0.0;
                    state.resolved_crash = Some(rc);
                }
                let rc = state.resolved_crash.unwrap();
                let pre = rc.internal;
                let f_base = rc.base_freq * rc.shimmer;
                let decay_rate = rc.decay_rate;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let brightness_vel = rc.brightness * (0.55 + 0.85 * aggression * rc.velocity_tilt);
                // Hitting a cymbal harder does not just raise its level — it
                // drives more energy into the high modes and shortens the swell.
                let swell = (rc.swell * (1.35 - 0.65 * aggression)).max(0.0002);

                // ─── Swell × decay ───────────────────────────────────────────
                let swell_env = 1.0 - fast_exp(-t / swell);
                let decay_env = fast_exp(-t * decay_rate);

                // ─── Plate modes ─────────────────────────────────────────────
                let coupling_drive = state.crash_coupling_z * pre.coupling;
                let nyquist = info.sample_rate * 0.5;
                state.sync_modes(MODE_KIND_CRASH, 0xC2A5_0000, pre.mode_freq_scale, decay_rate, info.sample_rate, info.sample_index, &rc.mode_amp);
                let w = TAU * f_base * t;
                let mut shimmer_low = 0.0;
                let mut shimmer_high = 0.0;
                for i in 0..state.mode_active {
                    let k = state.mode_k[i];
                    let mode_freq = f_base * k;
                    let rolloff = ((nyquist * 0.92 - mode_freq) / (nyquist * 0.08)).clamp(0.0, 1.0);
                    if rolloff <= 0.0 {
                        continue;
                    }
                    let phase = w * k + coupling_drive + state.mode_j[i];
                    let voice = fast_sin(phase) * rc.mode_amp[i] * state.mode_env[i] * rolloff;
                    if i >= 8 { shimmer_high += voice } else { shimmer_low += voice }
                }
                let shimmer = shimmer_low + shimmer_high * brightness_vel;
                state.crash_coupling_z = shimmer;

                let plate =
                    state.crash_hp.process_hp(shimmer, info.sample_rate, pre.hp_cutoff, 0.7);

                // ─── Wash ────────────────────────────────────────────────────
                let white = fast_noise(
                    state.haas_index(info.sample_index),
                    state.noise_seed(0xC2A5_A11),
                );
                let wash_lo = state.crash_bp_low.process(white)
                    * fast_exp(-t * decay_rate * pre.wash_lo.3)
                    * pre.wash_lo.2;
                let wash_hi = state.crash_bp_high.process(white)
                    * fast_exp(-t * decay_rate * pre.wash_hi.3)
                    * pre.wash_hi.2;
                let wash = (wash_lo + wash_hi) * rc.wash;

                let mixed = (plate * pre.mix.0 + wash * pre.mix.1) * swell_env * onset_ramp(t);

                // ─── Tail darkening ──────────────────────────────────────────
                // Cutoff falls with the decay envelope, so the cymbal loses its
                // top long before it loses its level — exactly how a real plate
                // sheds energy.
                let cutoff = pre.lp_sweep.1 + (pre.lp_sweep.0 - pre.lp_sweep.1) * decay_env;
                let shaped = state.crash_lp.process_lp(mixed, info.sample_rate, cutoff, 0.7);

                tape_sat(shaped) * info.gain * CRASH_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // SHAKER — a cloud of impacts, not noise with an envelope.
            //
            // The ear hears granularity directly: what separates a maraca (a few
            // big seeds) from a cabasa (hundreds of tiny beads) is the DENSITY of
            // the impact cloud, not the filter. Two cheap mechanisms model it:
            //
            //   GRAIN   — the noise is amplitude-modulated by a sample-and-hold
            //             random signal held for `grain_period` samples. Short
            //             hold = fine sand; long hold = big rattling seeds.
            //   BURSTS  — a handful of jittered sub-hits inside the one stroke,
            //             because particles do not all land at the same instant.
            //
            // `tambourine` adds two high-Q rings that outlive the cloud exciting
            // them: the jingles.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Shaker { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let note_freq = if info.frequency > 20.0 { info.frequency } else { DRUM_REF_FREQ };
                if state.resolved_shaker.is_none() {
                    let rs = resolve_shaker(params.as_ref());
                    let pre = rs.internal;
                    // Unpitched: `tune`/`Fm` colour the bands, they do not
                    // transpose them.
                    let shift = rs.band_shift
                        * rs.tune
                        * (note_freq / DRUM_REF_FREQ).powf(pre.pitch_track);
                    state.shaker_bp_low.bandpass(info.sample_rate, pre.band_lo.0 * shift, pre.band_lo.1);
                    state.shaker_bp_high.bandpass(info.sample_rate, pre.band_hi.0 * shift, pre.band_hi.1);
                    state.shaker_shell_bp.bandpass(info.sample_rate, pre.shell.0 * shift, pre.shell.1);
                    state.shaker_jingle_1.bandpass(info.sample_rate, (pre.jingle_bp.0).0 * shift, (pre.jingle_bp.0).1);
                    state.shaker_jingle_2.bandpass(info.sample_rate, (pre.jingle_bp.1).0 * shift, (pre.jingle_bp.1).1);
                    state.resolved_shaker = Some(rs);
                }
                let rs = state.resolved_shaker.unwrap();
                let pre = rs.internal;
                let decay = rs.decay;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let bright_vel = rs.brightness * (0.55 + 0.85 * aggression * rs.velocity_tilt);
                // A hard shake throws the particles together — denser cloud,
                // faster onset. A soft one lets them spill.
                let density_vel = (rs.density * (0.60 + 0.75 * aggression)).clamp(0.0, 1.0);
                let attack_tau = (rs.attack_tau * (1.30 - 0.55 * aggression)).max(0.00005);

                // ─── Burst train — the macro grain ───────────────────────────
                let n = pre.bursts.max(1);
                let mut env = 0.0;
                let mut burst_start = 0.0;
                for k in 0..n {
                    if t >= burst_start {
                        let local = t - burst_start;
                        let level = pre.burst_falloff.powi(k as i32);
                        env += level
                            * fast_exp(-local * 3.0 / decay)
                            * (1.0 - fast_exp(-local / attack_tau));
                    }
                    let jit = state.phase_jitter(0x5AA4_0000 ^ (k as u64))
                        / (std::f64::consts::PI / 15.0); // [-1, 1]
                    burst_start += pre.burst_spacing * (1.0 + jit * pre.burst_jitter);
                }
                // Bursts overlap, so normalise by √n rather than n — matches how
                // uncorrelated impacts actually sum in level.
                env /= (n as f64).sqrt();

                // ─── Grain — the micro texture ───────────────────────────────
                // Sample-and-hold on a deterministic hash: free, reproducible,
                // and the hold length maps directly onto particle size.
                let idx = state.haas_index(info.sample_index);
                let hold = (pre.grain_period.max(1) as f64 * info.sample_rate / 48000.0).max(1.0)
                    as usize;
                let sh = fast_noise(idx / hold, state.noise_seed(0x5AA4_6241));
                let grain = (1.0 + pre.grain_depth * density_vel * sh).max(0.0);

                let white = fast_noise(idx, state.noise_seed(0x5AA4_A11)) * grain;

                let lo = state.shaker_bp_low.process(white)
                    * pre.band_lo.2
                    * fast_exp(-t * 3.0 * pre.band_lo.3 / decay);
                let hi = state.shaker_bp_high.process(white)
                    * pre.band_hi.2
                    * bright_vel
                    * fast_exp(-t * 3.0 * pre.band_hi.3 / decay);
                let shell = state.shaker_shell_bp.process(white) * pre.shell.2;
                let cloud = (lo + hi + shell) * env * onset_ramp(t);

                // ─── Jingles ─────────────────────────────────────────────────
                // Rung by the same particle cloud that drives the shells, but
                // with their own much slower decay — a tambourine's rings carry
                // long after the beads have stopped.
                let jingle = if rs.jingle > 0.0 {
                    let ring_env = fast_exp(-t * 3.0 * pre.jingle_decay / decay);
                    let exc = white * fast_exp(-t * 3.0 / decay);
                    (state.shaker_jingle_1.process(exc) + state.shaker_jingle_2.process(exc) * 0.8)
                        * ring_env
                        * rs.jingle
                        * onset_ramp(t)
                } else {
                    0.0
                };

                tape_sat(cloud + jingle) * info.gain * SHAKER_GAIN * pre.gain_trim
            }

            // ═════════════════════════════════════════════════════════════════════
            // COWBELL — struck metal with two deliberately unrelated partials.
            //
            // The 808 recipe is a pair of detuned squares (~540 and ~800 Hz — a
            // ratio near a fifth but pointedly not one) through a bandpass with a
            // fast attack and a two-stage decay. Real squares would alias badly
            // up here, so each partial is a soft-clipped sine instead: the shaper
            // supplies the odd harmonics that make it read as "square" while the
            // series still dies off fast enough to stay clean. The shaping drive
            // is scaled down on the upper partials, which both keeps their
            // harmonics under Nyquist and matches how struck metal actually
            // distributes its energy.
            // ═════════════════════════════════════════════════════════════════════
            OscType::Cowbell { params } => {
                let t = info.sample_index as f64 / info.sample_rate;

                let note_freq = if info.frequency > 20.0 { info.frequency } else { DRUM_REF_FREQ };
                if state.resolved_cowbell.is_none() {
                    let mut rc = resolve_cowbell(params.as_ref());
                    let pre = rc.internal;
                    rc.base_freq =
                        drum_register(pre.base_pitch, rc.tune, note_freq, pre.pitch_track);
                    let track = rc.base_freq / pre.base_pitch;
                    state.cowbell_bp.bandpass(info.sample_rate, pre.bp.0 * track, pre.bp.1);
                    state
                        .cowbell_click_bp
                        .bandpass(info.sample_rate, pre.click_bp.0 * track, pre.click_bp.1);
                    state.resolved_cowbell = Some(rc);
                }
                let rc = state.resolved_cowbell.unwrap();
                let pre = rc.internal;
                let f_base = rc.base_freq;
                let decay = rc.decay;

                let velocity = info.gain.clamp(0.0, 1.0);
                let aggression = vel_curve(velocity);
                let click_vel = 0.30 + 1.15 * aggression * rc.velocity_tilt;
                // Struck harder = driven further into the shaper = clangier.
                let shape_vel = 0.55 + 0.85 * aggression;

                let nyquist = info.sample_rate * 0.5;
                let mut partials = 0.0;
                for i in 0..4 {
                    let ratio = if i == 1 { rc.ratio } else { pre.partial_ratios[i] };
                    let id = 0xC0B0_0000 ^ (i as u64).wrapping_mul(0x9E37);
                    let freq = f_base * ratio * state.freq_jitter(id);
                    if freq >= nyquist * 0.9 {
                        continue;
                    }
                    let phase = TAU * freq * t + state.phase_jitter(id);
                    let s = fast_sin(phase);
                    // Cap the drive so the shaper's harmonic series stays under
                    // Nyquist: a partial at f driven to D generates content out
                    // to roughly 3·D·f.
                    let headroom = (nyquist * 0.85 / (freq * 3.0)).clamp(0.0, 1.0);
                    let d = pre.shape_drive * shape_vel * headroom / (1.0 + ratio * 0.6);
                    let shaped = if d > 0.01 { fast_tanh(s * (1.0 + d)) } else { s };
                    let env = fast_exp(-t * 3.0 * pre.partial_decays[i] / decay);
                    partials += shaped * pre.partial_amps[i] * env;
                }

                // The "cup": a resonant bump mixed against the dry partials
                // rather than replacing them — a straight bandpass at the body
                // frequency would gut the fundamental it is supposed to sit on.
                let cup = state.cowbell_bp.process(partials);
                let body = partials * 0.55 + cup * 0.90;

                // Two-stage amplitude: a fast initial snap over the main decay.
                let main_env = fast_exp(-t * 3.0 / decay);
                let snap_env = pre.snap.0 * fast_exp(-t / pre.snap.1);
                let voiced = body * (main_env + snap_env);

                // Mallet click.
                let click_excite = if info.sample_index < 4 {
                    let ramp = 1.0 - info.sample_index as f64 * 0.25;
                    ramp + fast_noise(info.sample_index, state.noise_seed(0xC0B0_1CC0)) * 0.45
                } else {
                    0.0
                };
                let click = state.cowbell_click_bp.process(click_excite)
                    * rc.attack_amount
                    * click_vel
                    * pre.click_gain;

                tape_sat((voiced + click) * pre.drive_out)
                    * info.gain
                    * COWBELL_GAIN
                    * pre.gain_trim
            }
        }
    }
}
