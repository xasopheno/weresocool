use crate::instrument::voice::{SampleInfo, Voice};
use rand::Rng;
use std::f64::consts::PI;

fn tau() -> f64 {
    PI * 2.0
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
        let rand_range = 0.5;
        let r: f64 = rand::thread_rng().gen_range(-rand_range, rand_range);

        self.calculate_current_phase(&info, r);

        self.phase.sin() * info.gain
    }

    pub fn calculate_current_phase(&mut self, info: &SampleInfo, rand: f64) {
        let frequency = if self.sound_to_silence() {
            self.past.frequency
        } else if info.index < info.portamento_length
            && !self.silence_to_sound()
            && !self.sound_to_silence()
        {
            self.past.frequency + (info.index as f64 * info.p_delta)
        } else {
            self.current.frequency
        };

        let gain = info.gain;
        let current_phase = if gain == 0.0 {
            0.0
        } else {
            ((info.factor * frequency) + self.phase + rand) % tau()
        };

        self.phase = current_phase;
    }
}
