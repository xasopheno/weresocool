use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub struct KarplusStrong {
    buffer: VecDeque<f64>,
    decay: f64,
}

impl KarplusStrong {
    pub fn init(pitch: f64, sample_rate: f64, decay: f64) -> Self {
        let min_pitch = 0.001; // Define your minimum pitch value.
        let pitch = if pitch < min_pitch { min_pitch } else { pitch };
        let length = (sample_rate / pitch).round() as usize;

        let buffer: VecDeque<f64> = (0..length)
            .map(|_| rand::random::<f64>() * 2.0 - 1.0)
            .collect();

        KarplusStrong { buffer, decay }
    }

    pub fn generate_sample(&mut self) -> f64 {
        // Buffer should always have >= 2 elements due to initialization
        // Use unwrap_or to handle unexpected edge cases gracefully
        let first = self.buffer.pop_front().unwrap_or(0.0);
        let second = *self.buffer.front().unwrap_or(&0.0);
        let new_sample = self.decay * 0.5 * (first + second);
        self.buffer.push_back(new_sample);

        // Debug check: buffer should never be empty after initialization
        debug_assert!(!self.buffer.is_empty(), "KarplusStrong buffer became empty");

        new_sample
    }
}
