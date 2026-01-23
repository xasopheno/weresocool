use crate::voice::{SampleInfo, Voice};
use std::f64::consts::PI;
use weresocool_ast::OscType;
use weresocool_shared::r_to_f64;

const TAU: f64 = PI * 2.0;

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

            // Drum synthesis - uses timing info for internal envelopes
            OscType::Kick { params } => {
                let t = info.sample_index as f64 / info.sample_rate; // Time in seconds
                let t_norm = info.sample_index as f64 / info.total_samples.max(1) as f64;

                // ═══════════════════════════════════════════════════════════════════
                // SPECTRUM CONTROLS (0-1 scale)
                // ═══════════════════════════════════════════════════════════════════
                // attack: soft/round (0) → hard/clicky (1) - controls click_amount, transient_curve
                // body:   thin (0) → thick/subby (1) - controls sub_amount, hump
                // tone:   dark (0) → bright (1) - controls harmonic_damping, click_freq
                // length: tight (0) → boomy (1) - controls amp_decay

                let attack_spec = params.as_ref().and_then(|p| p.attack.map(r_to_f64)).unwrap_or(0.5);
                let body_spec = params.as_ref().and_then(|p| p.body.map(r_to_f64)).unwrap_or(0.5);
                let tone_spec = params.as_ref().and_then(|p| p.tone.map(r_to_f64)).unwrap_or(0.5);
                let length_spec = params.as_ref().and_then(|p| p.length.map(r_to_f64)).unwrap_or(0.5);

                // ═══════════════════════════════════════════════════════════════════
                // SPECIFIC PARAMETERS (override spectrum mappings)
                // ═══════════════════════════════════════════════════════════════════

                // pitch_decay is now in SECONDS (time for pitch to drop to ~5%)
                // 909-style: 0.02-0.03s (fast click), 808-style: 0.05-0.07s
                let pitch_decay = params.as_ref()
                    .and_then(|p| p.pitch_decay.map(r_to_f64))
                    .unwrap_or(0.03);  // 30ms default (909-style)
                let pitch_range = params.as_ref()
                    .and_then(|p| p.pitch_range.map(r_to_f64))
                    .unwrap_or(3.0);
                // amp_decay is now in SECONDS (909-style: 0.1-0.3s, 808-style: 0.3-0.8s)
                let amp_decay = params.as_ref()
                    .and_then(|p| p.amp_decay.map(r_to_f64))
                    .unwrap_or(0.10 + length_spec * 0.15);  // tight=0.10s (100ms), boomy=0.25s (250ms)
                let sub_amount = params.as_ref()
                    .and_then(|p| p.sub_amount.map(r_to_f64))
                    .unwrap_or(0.1 + body_spec * 0.4);  // 0.1-0.5
                let click_amount = params.as_ref()
                    .and_then(|p| p.click_amount.map(r_to_f64))
                    .unwrap_or(0.05 + attack_spec * 0.45);  // 0.05-0.5
                let click_freq_mult = params.as_ref()
                    .and_then(|p| p.click_freq.map(r_to_f64))
                    .unwrap_or(5.0 + tone_spec * 7.0);  // 5-12
                let attack_amount = 0.1 + attack_spec * 0.4;  // Derived from attack spectrum
                let harmonic_damping = params.as_ref()
                    .and_then(|p| p.harmonic_damping.map(r_to_f64))
                    .unwrap_or(2.5 - tone_spec * 1.3);  // dark=2.5, bright=1.2
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(0.2);
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.5);
                let transient_curve = params.as_ref()
                    .and_then(|p| p.transient_curve.map(r_to_f64))
                    .unwrap_or(1.5 + attack_spec * 2.0);  // 1.5-3.5

                // Velocity-dependent spectral tilt: higher velocity = more high-freq content
                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.powf(0.5);  // sqrt for natural feel
                let click_amount_vel = click_amount * (0.5 + 0.5 * spectral_tilt * velocity_tilt + 0.5 * (1.0 - velocity_tilt));
                let sub_amount_vel = sub_amount * (1.2 - 0.4 * spectral_tilt * velocity_tilt);
                let harmonic2_vel = 0.3 * (0.5 + 0.5 * spectral_tilt * velocity_tilt);

                // Integrate frequency to get phase for pitch-enveloped sine
                // For freq(t) = f0 * (1 + A*e^(-kt)), phase = 2π * f0 * (t + A/k * (1 - e^(-kt)))
                // Use fixed kick frequency (~60 Hz) - real kicks are 40-80 Hz regardless of musical key
                let f0 = 60.0;
                // pitch_decay is in seconds, using -3.0 coefficient for ~95% decay at specified time
                let integrated_time = t + (pitch_range * pitch_decay / 3.0) * (1.0 - (-t * 3.0 / pitch_decay).exp());
                let kick_phase = TAU * f0 * integrated_time;

                // Fundamental and second harmonic with frequency-dependent damping
                // Using -3.0 coefficient gives ~95% decay at the specified time (in seconds)
                let amp_fundamental = (-t * 3.0 / amp_decay).exp();
                let amp_harmonic2 = (-t * 3.0 * harmonic_damping / amp_decay).exp();  // Faster decay for harmonics
                let amp_sub = (-t * 3.0 * 0.5 / amp_decay).exp();  // Slower decay for subs

                let fundamental = kick_phase.sin() * amp_fundamental;
                let harmonic2 = (kick_phase * 2.0).sin() * harmonic2_vel * amp_harmonic2;

                // Sub-harmonics for depth (creates the "chest thump")
                let sub1_phase = TAU * (f0 * 0.5) * integrated_time;
                let sub2_phase = TAU * (f0 * 0.333) * integrated_time;
                let sub1 = sub1_phase.sin() * 0.15 * sub_amount_vel / 0.2 * amp_sub;
                let sub2 = sub2_phase.sin() * 0.08 * sub_amount_vel / 0.2 * amp_sub;

                // Click component (brief high-frequency transient)
                let click_freq = f0 * click_freq_mult;
                let click_decay_rate = 200.0;  // Very fast decay
                let click = (TAU * click_freq * t).sin() * (-t * click_decay_rate).exp() * click_amount_vel;

                // Enhanced transient envelope (multi-stage: spike → dip → settle)
                let transient = transient_envelope(t, transient_curve, 0.1);

                // Attack transient phase (0-8ms noise burst) - still useful for texture
                let attack_duration = 0.008;
                let attack_env = if t < attack_duration {
                    (-t * 150.0).exp()
                } else {
                    0.0
                };
                let attack_noise = fast_noise(info.sample_index, 0xABCDEF0123456789) * attack_env * attack_amount;

                // Combine tonal components
                let tone = fundamental + harmonic2 + sub1 + sub2 + click + attack_noise;

                // Add multi-stage transient
                let output = tone + transient * attack_amount;

                output * info.gain * 8.0
            }

            OscType::Snare { params } => {
                let t = info.sample_index as f64 / info.sample_rate;
                let t_norm = info.sample_index as f64 / info.total_samples.max(1) as f64;

                // ═══════════════════════════════════════════════════════════════════
                // SPECTRUM CONTROLS (0-1 scale)
                // ═══════════════════════════════════════════════════════════════════
                // attack: soft (0) → cracking (1) - controls attack_amount, crack, shell_pitch_range
                // wires:  dry/woody (0) → sizzly (1) - controls wire_mix, wire_decay
                // tone:   dark (0) → bright (1) - controls head_damping_ratio
                // length: tight (0) → ringy (1) - controls shell_decay

                let attack_spec = params.as_ref().and_then(|p| p.attack.map(r_to_f64)).unwrap_or(0.5);
                let wires_spec = params.as_ref().and_then(|p| p.wires.map(r_to_f64)).unwrap_or(0.5);
                let tone_spec = params.as_ref().and_then(|p| p.tone.map(r_to_f64)).unwrap_or(0.5);
                let length_spec = params.as_ref().and_then(|p| p.length.map(r_to_f64)).unwrap_or(0.5);

                // ═══════════════════════════════════════════════════════════════════
                // SPECIFIC PARAMETERS (override spectrum mappings)
                // ═══════════════════════════════════════════════════════════════════

                // Shell decay: now in SECONDS (909-style: 0.10-0.20s, 808-style: 0.25-0.35s)
                let shell_decay = params.as_ref()
                    .and_then(|p| p.shell_decay.or(p.tone_decay).map(r_to_f64))
                    .unwrap_or(0.10 + length_spec * 0.10);  // tight=0.10s (100ms), ringy=0.20s (200ms)

                // Wire decay: now in SECONDS (909-style: 0.15-0.25s, 808-style: 0.20-0.35s)
                let wire_decay = params.as_ref()
                    .and_then(|p| p.wire_decay.or(p.noise_decay).map(r_to_f64))
                    .unwrap_or(0.15 + wires_spec * 0.10);  // dry=0.15s (150ms), sizzly=0.25s (250ms)

                // Wire mix: 0=all shell, 1=all wire. Default 0.5 (dry=0.3, sizzly=0.7)
                let wire_mix = params.as_ref()
                    .and_then(|p| p.wire_mix.or(p.noise_mix).map(r_to_f64))
                    .unwrap_or(0.3 + wires_spec * 0.4);

                // Shell tune: bottom head frequency ratio. Default 1.8
                let shell_tune = params.as_ref()
                    .and_then(|p| p.shell_tune.map(r_to_f64))
                    .unwrap_or(1.8);

                // Attack amount: noise burst intensity. Default 0.4 (soft=0.2, cracking=0.6)
                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack_amount.map(r_to_f64))
                    .unwrap_or(0.2 + attack_spec * 0.4);

                // Shell pitch envelope decay rate. Default 30
                let shell_pitch_decay = params.as_ref()
                    .and_then(|p| p.shell_pitch_decay.map(r_to_f64))
                    .unwrap_or(30.0);

                // Shell pitch range: how much pitch drops. Default 0.3 (soft=0.1, cracking=0.5)
                let shell_pitch_range = params.as_ref()
                    .and_then(|p| p.shell_pitch_range.map(r_to_f64))
                    .unwrap_or(0.1 + attack_spec * 0.4);

                // Head damping ratio: top/bottom decay ratio. Default 1.5 (dark=2.0, bright=1.2)
                let head_damping_ratio = params.as_ref()
                    .and_then(|p| p.head_damping_ratio.map(r_to_f64))
                    .unwrap_or(2.0 - tone_spec * 0.8);

                // Saturation amount. Default 0.15
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(0.15);

                // Velocity tilt: how much velocity affects spectrum. Default 0.5
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.5);

                // Crack amount: tonal transient intensity. Default 0.3 (soft=0.1, cracking=0.5)
                let crack_amount = params.as_ref()
                    .and_then(|p| p.crack.map(r_to_f64))
                    .unwrap_or(0.1 + attack_spec * 0.4);

                // ═══════════════════════════════════════════════════════════════════
                // SYNTHESIS (909-style snare)
                // ═══════════════════════════════════════════════════════════════════

                // Velocity-dependent spectral tilt
                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.powf(0.5);
                let wire_mix_vel = wire_mix * (0.7 + 0.6 * spectral_tilt * velocity_tilt);

                // ─────────────────────────────────────────────────────────────────────
                // SHELL: Two slightly detuned tones for thickness (909-style)
                // 909 uses ~180-200 Hz fundamental with inharmonic overtones
                // ─────────────────────────────────────────────────────────────────────
                let base_freq = 185.0;  // 909-style fundamental
                let shell_pitch_mult = 1.0 + shell_pitch_range * (-t * shell_pitch_decay).exp();

                // Slightly inharmonic ratios for thickness (not perfect octave)
                let shell_freq_1 = base_freq * shell_pitch_mult;
                let shell_freq_2 = base_freq * 1.71 * shell_pitch_mult;  // ~316 Hz (not octave)
                let shell_freq_3 = base_freq * 2.80 * shell_pitch_mult;  // ~518 Hz (adds body)

                // Frequency-dependent damping
                let shell_amp_1 = (-t * 3.0 / shell_decay).exp();
                let shell_amp_2 = (-t * 3.0 * 1.5 / shell_decay).exp();
                let shell_amp_3 = (-t * 3.0 * 2.5 / shell_decay).exp();

                let shell = (TAU * shell_freq_1 * t).sin() * 0.5 * shell_amp_1
                          + (TAU * shell_freq_2 * t).sin() * 0.35 * shell_amp_2
                          + (TAU * shell_freq_3 * t).sin() * 0.2 * shell_amp_3;

                // ─────────────────────────────────────────────────────────────────────
                // CRACK: The defining 909 transient - very fast, punchy, high-mid focus
                // This is what gives the 909 its "snap"
                // ─────────────────────────────────────────────────────────────────────
                let crack_decay = 0.003;  // 3ms - very fast
                let crack_amp = (-t * 3.0 / crack_decay).exp();

                // Multiple crack frequencies for richness (909 has complex transient)
                let crack_1 = (TAU * 900.0 * t).sin() * 0.6;   // Main crack frequency
                let crack_2 = (TAU * 1200.0 * t).sin() * 0.3;  // Upper harmonic
                let crack_3 = (TAU * 600.0 * t).sin() * 0.25;  // Lower body
                let crack = (crack_1 + crack_2 + crack_3) * crack_amp * crack_amount;

                // ─────────────────────────────────────────────────────────────────────
                // NOISE: White noise (brighter than pink) with fast attack envelope
                // Simulates snare wires - should be "snappy" not "hissy"
                // ─────────────────────────────────────────────────────────────────────
                // Use white noise (fast_noise) for brighter, crispier wire sound
                let wire_noise = fast_noise(info.sample_index, 0xDEADBEEFCAFEBABE);

                // Two-stage envelope: fast attack spike + slower tail
                let wire_attack_amp = (-t * 3.0 / 0.008).exp();  // 8ms fast attack
                let wire_tail_amp = (-t * 3.0 / wire_decay).exp();  // Longer tail
                let wire_env = wire_attack_amp * 0.7 + wire_tail_amp * 0.3;
                let wire = wire_noise * wire_env;

                // ─────────────────────────────────────────────────────────────────────
                // ATTACK TRANSIENT: Initial broadband noise burst
                // ─────────────────────────────────────────────────────────────────────
                let attack_amp = (-t * 3.0 / 0.002).exp();  // 2ms burst
                let attack_noise = fast_noise(info.sample_index, 0xFEDCBA9876543210) * attack_amp * attack_amount;

                // ─────────────────────────────────────────────────────────────────────
                // MIX: Crack is always present, shell/wire balance controlled by wire_mix
                // ─────────────────────────────────────────────────────────────────────
                let shell_component = shell * (1.0 - wire_mix_vel * 0.5);  // Shell reduced by wire mix
                let wire_component = (wire + attack_noise) * (0.3 + wire_mix_vel * 0.7);  // Wire boosted by wire mix
                let tone = shell_component + wire_component + crack;

                // Soft saturation for warmth
                let decay_progress = 1.0 - shell_amp_1;
                let saturation_drive = 1.0 + saturation_amount * decay_progress;
                let saturated = soft_saturate(tone, saturation_drive);

                saturated * info.gain * 8.0
            }

            OscType::HiHat { open, params } => {
                let t = info.sample_index as f64 / info.sample_rate;
                let t_norm = info.sample_index as f64 / info.total_samples.max(1) as f64;

                // ═══════════════════════════════════════════════════════════════════
                // SPECTRUM CONTROLS (0-1 scale)
                // ═══════════════════════════════════════════════════════════════════
                // attack: soft (0) → clicky (1) - controls attack_amount, pitch_drop
                // metal:  dull (0) → shimmery (1) - controls shimmer, brightness
                // length: choked (0) → open (1) - controls decay_rate

                let attack_spec = params.as_ref().and_then(|p| p.attack.map(r_to_f64)).unwrap_or(0.5);
                let metal_spec = params.as_ref().and_then(|p| p.metal.map(r_to_f64)).unwrap_or(0.5);
                let length_spec = params.as_ref().and_then(|p| p.length.map(r_to_f64)).unwrap_or(0.5);

                // ═══════════════════════════════════════════════════════════════════
                // SPECIFIC PARAMETERS (override spectrum mappings)
                // ═══════════════════════════════════════════════════════════════════

                // decay_rate: higher = faster decay (uses t_norm, so scales with note length)
                let decay_base = if *open { 5.0 } else { 20.0 };
                let decay_range = if *open { 5.0 } else { 15.0 };
                let decay_rate = params.as_ref()
                    .and_then(|p| p.decay_rate.map(r_to_f64))
                    .unwrap_or(decay_base + (1.0 - length_spec) * decay_range);

                let shimmer_mult = params.as_ref()
                    .and_then(|p| p.shimmer.map(r_to_f64))
                    .unwrap_or(15.0 + metal_spec * 10.0);  // 15-25

                let brightness = params.as_ref()
                    .and_then(|p| p.brightness.map(r_to_f64))
                    .unwrap_or(0.5 + metal_spec * 0.5);  // 0.5-1.0

                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack_amount.map(r_to_f64))
                    .unwrap_or(0.1 + attack_spec * 0.3);  // 0.1-0.4

                let pitch_drop = params.as_ref()
                    .and_then(|p| p.pitch_drop.map(r_to_f64))
                    .unwrap_or(0.01 + attack_spec * 0.02);  // 0.01-0.03

                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(0.08);

                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(0.4);

                // Velocity-dependent spectral tilt
                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.powf(0.5);
                let brightness_vel = brightness * (0.7 + 0.6 * spectral_tilt * velocity_tilt);

                // Research-based frequency ratios for metallic sound
                // Non-integer ratios = inharmonic = metallic quality
                // Per-mode decay: higher frequencies decay faster (realistic)
                // Use fixed hi-hat base frequency (~400 Hz * shimmer_mult = ~6-10 kHz range)
                // Cymbals have slight pitch drop as energy dissipates
                let pitch_drop_mult = 1.0 + pitch_drop * (-t_norm * 5.0).exp();
                let base_freq = 400.0 * shimmer_mult * pitch_drop_mult;
                let modes: [(f64, f64, f64); 6] = [
                    // (freq_ratio, amplitude, decay_multiplier)
                    (1.00,  0.15, 1.0),   // Fundamental (damped)
                    (1.32,  0.25, 1.3),   // Strong mid
                    (1.68,  0.30, 1.8),   // Strong mid-high
                    (2.04,  0.25 * brightness_vel, 3.0),   // Bright (faster decay)
                    (2.57,  0.15 * brightness_vel, 5.0),   // Very bright (fast decay)
                    (2.92,  0.10 * brightness_vel, 8.0),   // Ultra-high (very fast)
                ];

                let mut shimmer = 0.0;
                for (ratio, amp, decay_mult) in modes {
                    let mode_freq = base_freq * ratio;
                    let mode_decay = decay_rate * decay_mult;  // Per-mode decay
                    let mode_amp = (-t_norm * mode_decay).exp();
                    shimmer += (TAU * mode_freq * t).sin() * amp * mode_amp;
                }

                // Pink noise for "air" and texture (better than white noise)
                let noise = pink_noise(info.sample_index, 0xFEEDFACECAFED00D);
                let noise_amp = (-t_norm * decay_rate).exp();

                // Attack transient phase (0-5ms for crisp stick hit)
                let attack_duration = 0.005;
                let attack_env = if t < attack_duration {
                    (-t * 200.0).exp()
                } else {
                    0.0
                };
                let attack_noise = fast_noise(info.sample_index, 0x1234567890ABCDEF) * attack_env * attack_amount;

                // Mix shimmer and noise
                let tone = shimmer * 0.4 + noise * noise_amp * 0.6 + attack_noise;

                tone * info.gain * 6.0
            }
        }
    }
}
