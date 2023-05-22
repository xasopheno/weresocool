use crate::voice::{SampleInfo, Voice};
use num_rational::Rational64;
use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use std::f64::consts::PI;
use weresocool_shared::{r_to_f64, Settings};

const TAU: f64 = PI * 2.0;

fn random_offset() -> f64 {
    thread_rng().gen_range(-0.5, 0.5)
}

#[derive(Clone, Debug, PartialEq)]
pub struct KarplusStrong {
    buffer: VecDeque<f64>,
    decay: f64,
}

impl KarplusStrong {
    pub fn init(pitch: f64, sample_rate: f64, decay: f64) -> Self {
        let length = (sample_rate / pitch).round() as usize;
        let buffer: VecDeque<f64> = (0..length)
            .map(|_| rand::random::<f64>() * 2.0 - 1.0)
            .collect();
        KarplusStrong { buffer, decay }
    }

    pub fn generate_sample(&mut self) -> f64 {
        let first = self.buffer.pop_front().unwrap();
        let second = *self.buffer.front().unwrap();
        let new_sample = self.decay * 0.5 * (first + second);
        self.buffer.push_back(new_sample);
        new_sample
    }
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

    pub fn generate_sawtooth_sample(&mut self, info: SampleInfo) -> f64 {
        self.phase = self.calculate_current_phase(&info, 0.0);
        2.0 * (self.phase / TAU - 0.5_f64.floor()) * info.gain
    }

    pub fn generate_triangle_sample(&mut self, info: SampleInfo, pow: Option<Rational64>) -> f64 {
        self.phase = self.calculate_current_phase(&info, 0.0);
        let value = match pow {
            Some(p) => {
                let power = r_to_f64(p);
                (f64::powf(self.phase, power).sin().abs() * 2.0 - 1.0) / power
            }
            None => self.phase.sin().abs() * 2.0 - 1.0,
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
            ((TAU / Settings::global().sample_rate).mul_add(info.frequency, self.phase) + rand)
                % TAU
        }
    }
}
