use crate::voice::{SampleInfo, Voice};
use std::f64::consts::PI;
use weresocool_ast::OscType;
use weresocool_shared::{r_to_f64, Settings};

const TAU: f64 = PI * 2.0;

use rand::{thread_rng, Rng};
pub fn random_offset() -> f64 {
    thread_rng().gen_range(-0.5, 0.5)
}

impl Voice {
    pub fn calculate_current_phase(info: &SampleInfo, osc_type: &OscType, prev_phase: f64) -> f64 {
        let rand = if *osc_type == OscType::Noise {
            random_offset()
        } else {
            0.0
        };

        if info.gain == 0.0 {
            0.0
        } else {
            ((TAU / Settings::global().sample_rate).mul_add(info.frequency, prev_phase) + rand)
                % TAU
        }
    }
}
pub trait Waveform {
    fn generate_sample(&self, info: SampleInfo, phase: f64) -> f64;
}

impl Waveform for OscType {
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
                let rate_factor = TAU / Settings::global().sample_rate;

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
        }
    }
}
