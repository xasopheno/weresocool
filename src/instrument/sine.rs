use instrument::voice::{SampleInfo, Voice, ASR};
use std::f64::consts::PI;

fn tau() -> f64 {
    PI * 2.0
}

impl Voice {
    pub fn generate_sine_sample(&mut self, info: SampleInfo) -> f64 {
        let frequency =
            if self.sound_to_silence() {
            self.current.frequency
        } else if info.index < info.portamento_length
            && !self.silence_to_sound()
            && !self.sound_to_silence()
        {
            self.past.frequency + (info.index as f64 * info.p_delta)
        } else {
            self.current.frequency
        };

        let gain = info.g_delta;
        let mut current_phase = ((info.factor * frequency) + self.phase) % tau();
        if gain == 0.0 {
            current_phase = 0.0;
        }
        self.phase = current_phase;

        current_phase.sin() * gain
    }
}
