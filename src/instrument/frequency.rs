use crate::instrument::voice::Voice;

impl Voice {
    pub fn calculate_portamento_delta(
        &self,
        portamento_length: usize,
        start: f64,
        target: f64,
    ) -> f64 {
        (target - start) / (portamento_length as f64)
    }

    pub fn calculate_frequency(
        &self,
        index: usize,
        portamento_length: usize,
        p_delta: f64,
        start: f64,
        target: f64,
    ) -> f64 {
        if self.sound_to_silence() {
            start
        } else if index < portamento_length && !self.silence_to_sound() && !self.sound_to_silence()
        {
            start + index as f64 * p_delta
        } else {
            target
        }
    }
}
