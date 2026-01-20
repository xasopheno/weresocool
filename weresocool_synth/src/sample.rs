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

                // Meta-parameters (0-1 scale, default 0.5, can exceed 1 to push)
                let punch = params.as_ref().and_then(|p| p.punch.map(r_to_f64)).unwrap_or(0.5);
                let body = params.as_ref().and_then(|p| p.body.map(r_to_f64)).unwrap_or(0.5);
                let air = params.as_ref().and_then(|p| p.air.map(r_to_f64)).unwrap_or(0.5);
                let dynamics = params.as_ref().and_then(|p| p.dynamics.map(r_to_f64)).unwrap_or(0.5);

                // Derive specific params from meta-params, allow override
                // punch → click_amount, attack, transient_curve
                // body → sub_amount, saturation, amp_decay (inverse - more body = slower decay)
                // air → harmonic_damping (inverse - more air = less damping), brightness
                // dynamics → velocity_tilt

                let pitch_decay = params.as_ref()
                    .and_then(|p| p.pitch_decay.map(r_to_f64))
                    .unwrap_or(50.0);
                let pitch_range = params.as_ref()
                    .and_then(|p| p.pitch_range.map(r_to_f64))
                    .unwrap_or(3.0);
                let amp_decay = params.as_ref()
                    .and_then(|p| p.amp_decay.map(r_to_f64))
                    .unwrap_or(6.0 + (1.0 - body) * 4.0);  // body: more = slower (6-10)
                let sub_amount = params.as_ref()
                    .and_then(|p| p.sub_amount.map(r_to_f64))
                    .unwrap_or(body * 0.4);  // body: 0-0.4
                let click_amount = params.as_ref()
                    .and_then(|p| p.click_amount.map(r_to_f64))
                    .unwrap_or(punch * 0.5 + air * 0.1);  // punch + air contribution
                let click_freq_mult = params.as_ref()
                    .and_then(|p| p.click_freq.map(r_to_f64))
                    .unwrap_or(8.0);
                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack.map(r_to_f64))
                    .unwrap_or(punch * 0.6);  // punch: 0-0.6
                let harmonic_damping = params.as_ref()
                    .and_then(|p| p.harmonic_damping.map(r_to_f64))
                    .unwrap_or(1.0 + (1.0 - air) * 1.5);  // air: less = more damping (1.0-2.5)
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(body * 0.5);  // body: 0-0.5
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(dynamics);  // dynamics maps directly
                let transient_curve = params.as_ref()
                    .and_then(|p| p.transient_curve.map(r_to_f64))
                    .unwrap_or(1.0 + punch * 2.0);  // punch: 1-3

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
                let integrated_time = t + (pitch_range / pitch_decay) * (1.0 - (-pitch_decay * t).exp());
                let kick_phase = TAU * f0 * integrated_time;

                // Fundamental and second harmonic with frequency-dependent damping
                let amp_fundamental = (-t_norm * amp_decay).exp();
                let amp_harmonic2 = (-t_norm * amp_decay * harmonic_damping).exp();  // Faster decay for harmonics
                let amp_sub = (-t_norm * amp_decay * 0.5).exp();  // Slower decay for subs

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

                // Meta-parameters (0-1 scale, default 0.5, can exceed 1 to push)
                let punch = params.as_ref().and_then(|p| p.punch.map(r_to_f64)).unwrap_or(0.5);
                let body = params.as_ref().and_then(|p| p.body.map(r_to_f64)).unwrap_or(0.5);
                let air = params.as_ref().and_then(|p| p.air.map(r_to_f64)).unwrap_or(0.5);
                let dynamics = params.as_ref().and_then(|p| p.dynamics.map(r_to_f64)).unwrap_or(0.5);

                // Derive specific params from meta-params, allow override
                // punch → attack, shell_pitch_range
                // body → saturation, shell_decay (inverse - more body = longer ring)
                // air → wire_mix (more air = more wires/brightness), head_damping_ratio (inverse)
                // dynamics → velocity_tilt

                let tone_pitch_decay = params.as_ref()
                    .and_then(|p| p.pitch_decay.map(r_to_f64))
                    .unwrap_or(80.0);
                let tone_pitch_range = params.as_ref()
                    .and_then(|p| p.pitch_range.map(r_to_f64))
                    .unwrap_or(2.0);
                let shell_decay = params.as_ref()
                    .and_then(|p| p.shell_decay.or(p.tone_decay).map(r_to_f64))
                    .unwrap_or(8.0 + body * 8.0);  // body: 8-16
                let wire_decay = params.as_ref()
                    .and_then(|p| p.wire_decay.or(p.noise_decay).map(r_to_f64))
                    .unwrap_or(20.0);
                let wire_mix = params.as_ref()
                    .and_then(|p| p.wire_mix.or(p.noise_mix).map(r_to_f64))
                    .unwrap_or(0.4 + air * 0.4);  // air: 0.4-0.8
                let shell_tune = params.as_ref()
                    .and_then(|p| p.shell_tune.map(r_to_f64))
                    .unwrap_or(1.8);
                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack.map(r_to_f64))
                    .unwrap_or(punch * 0.8);  // punch: 0-0.8
                let shell_pitch_decay = params.as_ref()
                    .and_then(|p| p.shell_pitch_decay.map(r_to_f64))
                    .unwrap_or(30.0);
                let shell_pitch_range = params.as_ref()
                    .and_then(|p| p.shell_pitch_range.map(r_to_f64))
                    .unwrap_or(punch * 0.5);  // punch: 0-0.5
                let head_damping_ratio = params.as_ref()
                    .and_then(|p| p.head_damping_ratio.map(r_to_f64))
                    .unwrap_or(1.0 + (1.0 - air) * 1.0);  // air: less = more damping (1.0-2.0)
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(body * 0.4);  // body: 0-0.4
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(dynamics);  // dynamics maps directly

                // Velocity-dependent spectral tilt
                let velocity = info.gain.clamp(0.0, 1.0);
                let spectral_tilt = velocity.powf(0.5);
                let wire_mix_vel = wire_mix * (0.7 + 0.6 * spectral_tilt * velocity_tilt);  // More wires at high vel

                // Shell component: dual-mode for top/bottom head resonance with pitch glide
                // Shell pitch drops during decay (membrane tension relaxes)
                // Use fixed snare frequency (~180 Hz base) - real snares are 150-250 Hz
                let base_freq = 180.0;
                let shell_pitch_mult = 1.0 + shell_pitch_range * (-t * shell_pitch_decay).exp();
                let shell_freq_1 = base_freq * 1.4 * shell_pitch_mult;   // Top head (~250 Hz)
                let shell_freq_2 = base_freq * shell_tune * shell_pitch_mult;  // Bottom head (lower)

                // Integrate for pitch envelope on body tone
                let integrated_time = t + (tone_pitch_range / tone_pitch_decay)
                    * (1.0 - (-tone_pitch_decay * t).exp());

                let shell_phase_1 = TAU * shell_freq_1 * integrated_time;
                let shell_phase_2 = TAU * shell_freq_2 * integrated_time;

                // Frequency-dependent damping: top head (higher freq) decays faster
                let shell_amp_1 = (-t_norm * shell_decay * (head_damping_ratio.sqrt())).exp();  // Top head: faster
                let shell_amp_2 = (-t_norm * shell_decay / (head_damping_ratio.sqrt())).exp();  // Bottom head: slower
                let shell = shell_phase_1.sin() * 0.6 * shell_amp_1
                          + shell_phase_2.sin() * 0.4 * shell_amp_2;

                // Wire component: pink noise (band-limited) for snare wire sound
                let wire_noise = pink_noise(info.sample_index, 0xDEADBEEFCAFEBABE);
                let wire_amp = (-t_norm * wire_decay).exp();
                let wire = wire_noise * wire_amp;

                // Attack transient phase (0-8ms noise burst)
                let attack_duration = 0.008;
                let attack_env = if t < attack_duration {
                    (-t * 150.0).exp()
                } else {
                    0.0
                };
                let attack_noise = fast_noise(info.sample_index, 0xFEDCBA9876543210) * attack_env * attack_amount;

                // Mix shell and wire components
                let tone = shell * (1.0 - wire_mix_vel) + wire * wire_mix_vel + attack_noise;

                tone * info.gain * 8.0
            }

            OscType::HiHat { open, params } => {
                let t = info.sample_index as f64 / info.sample_rate;
                let t_norm = info.sample_index as f64 / info.total_samples.max(1) as f64;

                // Meta-parameters (0-1 scale, default 0.5, can exceed 1 to push)
                let punch = params.as_ref().and_then(|p| p.punch.map(r_to_f64)).unwrap_or(0.5);
                let body = params.as_ref().and_then(|p| p.body.map(r_to_f64)).unwrap_or(0.5);
                let air = params.as_ref().and_then(|p| p.air.map(r_to_f64)).unwrap_or(0.5);
                let dynamics = params.as_ref().and_then(|p| p.dynamics.map(r_to_f64)).unwrap_or(0.5);

                // Derive specific params from meta-params, allow override
                // punch → attack
                // body → saturation, decay (inverse - more body = longer sustain)
                // air → brightness, shimmer
                // dynamics → velocity_tilt

                let default_decay = if *open { 4.0 } else { 25.0 };
                let decay_base = if *open { 3.0 } else { 15.0 };
                let decay_range = if *open { 4.0 } else { 20.0 };
                let decay_rate = params.as_ref()
                    .and_then(|p| p.decay.map(r_to_f64))
                    .unwrap_or(decay_base + (1.0 - body) * decay_range);  // body: more = slower decay
                let shimmer_mult = params.as_ref()
                    .and_then(|p| p.shimmer.map(r_to_f64))
                    .unwrap_or(15.0 + air * 10.0);  // air: 15-25
                let brightness = params.as_ref()
                    .and_then(|p| p.brightness.map(r_to_f64))
                    .unwrap_or(0.5 + air);  // air: 0.5-1.5
                let attack_amount = params.as_ref()
                    .and_then(|p| p.attack.map(r_to_f64))
                    .unwrap_or(punch * 0.4);  // punch: 0-0.4
                let pitch_drop = params.as_ref()
                    .and_then(|p| p.pitch_drop.map(r_to_f64))
                    .unwrap_or(0.02);
                let saturation_amount = params.as_ref()
                    .and_then(|p| p.saturation.map(r_to_f64))
                    .unwrap_or(body * 0.2);  // body: 0-0.2
                let velocity_tilt = params.as_ref()
                    .and_then(|p| p.velocity_tilt.map(r_to_f64))
                    .unwrap_or(dynamics);  // dynamics maps directly

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
