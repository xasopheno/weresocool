use crate::instrument::voice::{SampleInfo, Voice};
use rand::{thread_rng, Rng};
use std::f64::consts::PI;

fn tau() -> f64 {
    PI * 2.0
}

fn random_offset() -> f64 {
    thread_rng().gen_range(-0.5, 0.5)
}

impl Voice {
    pub fn generate_sine_sample(&mut self, info: SampleInfo) -> f64 {
        self.calculate_current_phase(&info, 0.0);

        self.phase.sin() * info.gain
    }

    pub fn generate_square_sample(&mut self, info: SampleInfo) -> f64 {
        self.calculate_current_phase(&info, 0.0);

        let s = self.phase.sin();
        s.signum() * info.gain
    }

    pub fn generate_random_sample(&mut self, info: SampleInfo) -> f64 {
        self.calculate_current_phase(&info, random_offset());

        self.phase.sin() * info.gain
    }

    pub fn calculate_current_phase(&mut self, info: &SampleInfo, rand: f64) {
        let current_phase = if info.gain == 0.0 {
            0.0
        } else {
            ((info.factor * info.frequency) + self.phase + rand) % tau()
        };

        self.phase = current_phase;
    }
}
