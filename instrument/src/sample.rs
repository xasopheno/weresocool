use crate::voice::{SampleInfo, Voice};
use num_rational::Rational64;
use std::f64::consts::PI;
use weresocool_shared::{r_to_f64, Settings};

const TAU: f64 = PI * 2.0;

impl Voice {
    pub fn generate_sine_sample(&self, info: SampleInfo, pow: Option<Rational64>) -> f64 {
        let value = match pow {
            Some(p) => {
                let power = r_to_f64(p);
                f64::powf(self.phase, power).sin() / power
            }
            None => self.phase.sin(),
        };
        value * info.gain
    }

    pub fn generate_sawtooth_sample(&self, info: SampleInfo) -> f64 {
        2.0 * (self.phase / TAU - 0.5_f64.floor()) * info.gain
    }

    pub fn generate_triangle_sample(&self, info: SampleInfo, pow: Option<Rational64>) -> f64 {
        let value = match pow {
            Some(p) => {
                let power = r_to_f64(p);
                (f64::powf(self.phase, power).sin().abs() * 2.0 - 1.0) / power
            }
            None => self.phase.sin().abs() * 2.0 - 1.0,
        };
        value * info.gain
    }

    pub fn generate_square_sample(&self, info: SampleInfo, width: Option<Rational64>) -> f64 {
        let pulse_width = if let Some(w) = width {
            r_to_f64(w)
        } else {
            0.0
        };

        let s = self.phase.sin();
        let sign = if s > pulse_width { -1. } else { 1. };
        sign * info.gain
    }

    pub fn generate_random_sample(&self, info: SampleInfo) -> f64 {
        self.phase.sin() * info.gain
    }

    pub fn calculate_current_phase(&self, info: &SampleInfo, rand: f64) -> f64 {
        if info.gain == 0.0 {
            0.0
        } else {
            ((TAU / Settings::global().sample_rate).mul_add(info.frequency, self.phase) + rand)
                % TAU
        }
    }
}
