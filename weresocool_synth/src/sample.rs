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
    fn generate_sample(&self, info: SampleInfo, phase: f64) -> f64;
}

impl Waveform for OscType {
    #[inline]
    fn generate_sample(&self, info: SampleInfo, phase: f64) -> f64 {
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

                // ─── Pitch-envelope phase integration ─────────────────────────
                //   freq(t) = f_base * (1 + (R-1) * exp(-t/τ))
                //   τ = pitch_decay / 3 → ~95% settled at t = pitch_decay
                let tau_p = pitch_decay / 3.0;
                let exp_pd = (-t / tau_p).exp();
                let env_factor = (pitch_range - 1.0) * tau_p * (1.0 - exp_pd);
                let kick_phase = TAU * f_base * (t + env_factor);

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

                // ─── Click — short noisy-sine in the transient register ───────
                // The click is a sine + a tiny noise burst on top, gated by a
                // 3 ms envelope. The noise gives the click "beater" character
                // instead of just being a clean tone.
                let click_decay = 0.0025;
                let click_env = (-t / click_decay).exp();
                let click_noise = bright_noise(info.sample_index, 0xABCDEF0123456789) * 0.35;
                let click = ((TAU * click_freq * t).sin() + click_noise)
                          * click_env * click_amount_vel;

                // ─── Saturation — proper tanh, NOT level-normalised ───────────
                // Drive scales with envelope so the loud part gets fat odd
                // harmonics while the tail stays clean. Output level boost via
                // (1 + sat) keeps perceived loudness consistent across sat
                // settings.
                let body = fundamental * body_env;
                let drive = 1.0 + saturation_amount * 2.5 * body_decay_env;
                let body_sat = (body * drive).tanh() * (1.0 + saturation_amount * 0.4);

                (body_sat + click) * info.gain * KICK_GAIN
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

                let head = (TAU * top_f1 * t).sin() * 0.42 * top_amp1
                         + (TAU * top_f2 * t).sin() * 0.22 * top_amp2
                         + (TAU * bot_f1 * t).sin() * 0.30 * bot_amp1
                         + (TAU * bot_f2 * t).sin() * 0.16 * bot_amp2;

                // ─── Wires: bright bandpassed noise driven by the heads ───────
                // The wire envelope has its OWN decay tail but is also gated
                // by the bottom-head envelope (sympathetic drive). This is
                // what makes the wires "rattle along" instead of hissing as a
                // separate layer. Use bright_noise + a comb stage centred
                // around 6-7 kHz for the proper "tsss" texture.
                let raw_wire_noise = bright_noise(info.sample_index, 0xDEADBEEFCAFEBABE) * 0.5
                                   + comb_noise(info.sample_index, 0xDEADBEEFCAFEBABE, 4) * 0.5;
                let wire_attack = (-t * 3.0 / 0.006).exp();           // 6 ms snap
                let wire_tail = (-t * 3.0 / wire_decay).exp();        // longer tail
                // Sympathetic drive: when bot_amp1 dies, wires die with it.
                let sympathy = 0.4 + 0.6 * bot_amp1;
                let wire_env = (wire_attack * 0.7 + wire_tail * 0.3) * sympathy;
                let wires = raw_wire_noise * wire_env;

                // ─── Crack: noise burst, not pure sines ───────────────────────
                // Real cracks are stick-on-head impacts — broadband noise with
                // a fast envelope, not tonal. Two stages: a *very* fast attack
                // spike (~1 ms, comb-shaped around 4-6 kHz) and a slightly
                // slower body (~5 ms, broader). Crack peaks just above unity
                // to compete with the head modes on the attack.
                let crack_spike_env = (-t / 0.0010).exp();
                let crack_body_env = (-t / 0.0050).exp();
                let crack_spike = (comb_noise(info.sample_index, 0x123456789ABCDEF0, 6) * 1.4
                                +  bright_noise(info.sample_index, 0x123456789ABCDEF0) * 0.6)
                                * crack_spike_env;
                let crack_body = comb_noise(info.sample_index, 0xCAFEBABEFEEDFACE, 10) * crack_body_env;
                let crack = (crack_spike + crack_body * 0.5) * crack_amount * 1.6;

                // ─── Beater click — ultra-short stick contact ─────────────────
                let beater_env = (-t / 0.0012).exp();
                let beater = bright_noise(info.sample_index, 0xFEDCBA9876543210)
                           * beater_env * attack_amount * 1.4;

                // ─── Mix + saturation ─────────────────────────────────────────
                // Head fades back slightly as wire content rises (real snares
                // have wires masking the head ring at high wire_mix).
                let head_component = head * (1.0 - wire_mix_vel * 0.35);
                let wire_component = wires * (0.4 + wire_mix_vel * 0.9);
                let tone = head_component + wire_component + crack + beater;

                let decay_progress = 1.0 - top_amp1;
                let drive = 1.0 + saturation_amount * (1.0 + decay_progress);
                let saturated = soft_saturate(tone, drive);

                saturated * info.gain * SNARE_GAIN
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

                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.sqrt();
                let brightness_vel = brightness * (0.7 + 0.6 * spectral_tilt * velocity_tilt);

                // Slight pitch droop — cymbals lose high-end energy first.
                let pitch_drop_mult = 1.0 + pitch_drop * (-t * 6.0).exp();
                let base_freq = info_freq * tune * shimmer_mult * pitch_drop_mult;

                // Mode set inspired by the first few Bessel-function zeros for
                // a circular plate (cymbal physics) — plus a few intentionally
                // close-pair detunings to create *beating*. Close pairs (e.g.
                // 1.594 + 1.612) interfere as `cos(2π*Δf*t)`, producing the
                // slow shimmer modulation we hear on real cymbals. Without
                // beating, the modes just stack into a chord.
                let modes: [(f64, f64, f64); 12] = [
                    (1.000, 0.16, 1.0),
                    (1.594, 0.22, 1.4),
                    (1.612, 0.16, 1.5),      // beats with 1.594
                    (2.135, 0.20, 2.0),
                    (2.295, 0.22, 2.3),
                    (2.310, 0.14, 2.4),      // beats with 2.295
                    (2.653, 0.18 * brightness_vel, 2.9),
                    (2.917, 0.16 * brightness_vel, 3.6),
                    (3.156, 0.14 * brightness_vel, 4.3),
                    (3.500, 0.11 * brightness_vel, 5.2),
                    (3.598, 0.09 * brightness_vel, 5.8),  // beats with 3.500
                    (4.060, 0.07 * brightness_vel, 7.0),
                ];

                let mut shimmer = 0.0;
                for (ratio, amp, decay_mult) in modes {
                    let mode_freq = base_freq * ratio;
                    // Skip aliased modes — anything above Nyquist would fold back.
                    if mode_freq >= info.sample_rate * 0.5 { continue; }
                    let mode_amp = (-t * decay_rate * decay_mult).exp();
                    shimmer += (TAU * mode_freq * t).sin() * amp * mode_amp;
                }

                // ─── Metallic noise — comb-filtered, modally gated ────────────
                // metal_noise() emphasises the 6-12 kHz band. We then gate it
                // by the slowest-decaying mode amplitude so the noise tracks
                // the metal — when the cymbal energy fades, the noise fades.
                let air = metal_noise(info.sample_index, 0xFEEDFACECAFED00D);
                let air_env = (-t * decay_rate * 1.1).exp();
                let air_signal = air * air_env;

                // ─── Sharp attack — stick contact ─────────────────────────────
                let attack_env = (-t / 0.0015).exp();
                let attack_noise =
                    bright_noise(info.sample_index, 0x1234567890ABCDEF) * attack_env * attack_amount;

                // Mix — modes carry slightly less than noise so the result is
                // perceptibly noisy/metallic rather than pitched.
                let tone = shimmer * 0.45 + air_signal * 0.55 + attack_noise;

                tone * info.gain * HIHAT_GAIN
            }
        }
    }
}
