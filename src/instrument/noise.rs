use instrument::voice::{SampleInfo, Voice};
use rand::Rng;
use std::f64::consts::PI;
fn tau() -> f64 {
    PI * 2.0
}

impl Voice {
    pub fn generate_random_sample(&mut self, info: SampleInfo) -> f64 {
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
        let gain = (info.index as f64 * info.g_delta) + self.past.gain;

        let rand_range = 0.5;
        let r: f64 = rand::thread_rng().gen_range(-rand_range, rand_range);

        let current_phase = ((info.factor * frequency) + self.phase + r) % tau();
        self.phase = current_phase;

        current_phase.sin() * gain
    }
}
