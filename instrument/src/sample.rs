use crate::voice::{SampleInfo, Voice};
use num_rational::Rational64;
use rand::{thread_rng, Rng};
use std::f64::consts::PI;
use weresocool_shared::{default_settings, r_to_f64, Settings};

const SETTINGS: Settings = default_settings();
const TAU: f64 = PI * 2.0;
const FACTOR: f64 = TAU / SETTINGS.sample_rate;

fn random_offset() -> f64 {
    thread_rng().gen_range(-0.5..0.5)
}

impl Voice {
    pub fn generate_sine_sample(&mut self, info: SampleInfo, pow: Option<Rational64>) -> f64 {
        self.phase = self.calculate_current_phase(&info, 0.0);
        let value = match pow {
            Some(p) => {
                let power = r_to_f64(p);
                f64::powf(self.phase, power).sin() / power
            }
            None => self.phase.sin(),
        };
        value * info.gain
    }

    pub fn generate_square_sample(&mut self, info: SampleInfo, width: Option<Rational64>) -> f64 {
        self.phase = self.calculate_current_phase(&info, 0.0);
        let pulse_width = if let Some(w) = width {
            r_to_f64(w)
        } else {
            0.0
        };

        let s = self.phase.sin();
        let sign = if s > pulse_width { -1. } else { 1. };
        sign * info.gain
    }

    pub fn generate_random_sample(&mut self, info: SampleInfo) -> f64 {
        self.phase = self.calculate_current_phase(&info, random_offset());

        self.phase.sin() * info.gain
    }

    pub fn calculate_current_phase(&mut self, info: &SampleInfo, rand: f64) -> f64 {
        if info.gain == 0.0 {
            0.0
        } else {
            (FACTOR.mul_add(info.frequency, self.phase) + rand) % TAU
        }
    }
}
