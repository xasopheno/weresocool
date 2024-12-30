pub mod fourier;

pub trait Analyze {
    fn yin_pitch_detection(&mut self, sample_rate: f32, threshold: f32) -> (f32, f32);
    fn get_better_tau(&self, tau: usize, sample_rate: f32) -> f32;
    fn yin_difference(&mut self);
    fn yin_absolute_threshold(&self, threshold: f32) -> Option<usize>;
    fn yin_parabolic_interpolation(&self, tau_estimate: usize) -> f32;
    fn yin_cumulative_mean_normalized_difference(&mut self);
    fn gain(&self) -> f32;
    fn analyze(&mut self, sample_rate: f32, threshold: f32) -> DetectionResult;
}

pub struct DetectionResult {
    pub frequency: f32,
    pub probability: f32,
    pub gain: f32,
}

impl Analyze for Vec<f32> {
    fn analyze(&mut self, sample_rate: f32, threshold: f32) -> DetectionResult {
        let gain = self.gain();
        let (frequency, probability) = self.yin_pitch_detection(sample_rate, threshold);

        DetectionResult {
            frequency,
            probability,
            gain,
        }
    }

    fn gain(&self) -> f32 {
        let sum_of_squares: f32 = self.iter().map(|&x| x.powi(2)).sum();
        let root_sum_of_squares = sum_of_squares.sqrt() / 100.0;
        root_sum_of_squares.min(1.0)
    }

    fn yin_pitch_detection(&mut self, sample_rate: f32, threshold: f32) -> (f32, f32) {
        // Scale samples
        for sample in self.iter_mut() {
            *sample *= 1000.0;
        }

        self.yin_difference();
        self.yin_cumulative_mean_normalized_difference();

        if let Some(tau) = self.yin_absolute_threshold(threshold) {
            let frequency = self.get_better_tau(tau, sample_rate);
            let probability = 1.0 - self[tau];
            (frequency, probability)
        } else {
            (0.0, -1.0)
        }
    }

    fn get_better_tau(&self, tau: usize, sample_rate: f32) -> f32 {
        let better_tau = self.yin_parabolic_interpolation(tau);
        sample_rate / better_tau
    }

    fn yin_difference(&mut self) {
        let buffer_clone = self.clone();
        let half_buffer_size = self.len() / 2;

        for tau in 0..half_buffer_size {
            for i in 0..half_buffer_size {
                let delta: f32 = buffer_clone[i] - buffer_clone[i + tau];
                self[tau] += delta * delta;
            }
        }

        self.resize(half_buffer_size, 0.0)
    }

    fn yin_cumulative_mean_normalized_difference(&mut self) {
        let mut running_sum = 0.0;

        for (tau, value) in self.iter_mut().enumerate() {
            running_sum += *value;
            if running_sum != 0.0 {
                *value *= tau as f32 / running_sum;
            } else {
                *value = 0.0;
            }
        }
    }

    fn yin_absolute_threshold(&self, threshold: f32) -> Option<usize> {
        let len = self.len();
        for tau in 2..len {
            if self[tau] < threshold {
                let mut tau_min = tau;
                while tau_min + 1 < len && self[tau_min + 1] < self[tau_min] {
                    tau_min += 1;
                }
                return Some(tau_min);
            }
        }
        Some(len - 1)
    }

    fn yin_parabolic_interpolation(&self, tau_estimate: usize) -> f32 {
        let x0 = if tau_estimate > 0 {
            tau_estimate - 1
        } else {
            tau_estimate
        };
        let x2 = if tau_estimate + 1 < self.len() {
            tau_estimate + 1
        } else {
            tau_estimate
        };

        let s0 = self[x0];
        let s1 = self[tau_estimate];
        let s2 = self[x2];

        let denom = 2.0 * (2.0 * s1 - s2 - s0);
        if denom.abs() < f32::EPSILON {
            tau_estimate as f32
        } else {
            tau_estimate as f32 + (s2 - s0) / denom
        }
    }
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
mod tests {
    use super::*;
    use weresocool_shared::helpers::{cmp_f32, cmp_vec_f32};

    #[test]
    fn gain_test() {
        let mut buffer = vec![
            0.0, 0.06279052, 0.12533323, 0.18738133, 0.2486899, 0.309017, 0.36812457, 0.4257793,
            0.4817537, 0.53582686,
        ];
        let gain = buffer.gain();
        let expected = 0.0102376845;
        assert!(cmp_f32(gain, expected));
    }

    #[test]
    fn difference_test() {
        let mut buffer = vec![
            0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0,
            -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0,
        ];
        let expected = vec![
            0.0, 4.25, 11.5, 16.75, 22.0, 20.25, 13.5, 6.75, 2.0, 1.75, 7.75, 15.75, 21.75, 21.75,
            15.75, 7.75,
        ];
        buffer.yin_difference();
        assert!(cmp_vec_f32(buffer, expected));
    }

    #[test]
    fn cumulative_mean_normalized_difference_test() {
        let mut buffer = vec![
            0.0, 4.25, 11.5, 16.75, 22.0, 20.25, 13.5, 6.75, 2.0, 1.75, 7.75, 15.75, 21.75, 21.75,
            15.75, 7.75,
        ];
        let expected = [
            0.0, 1.0, 1.4603175, 1.5461539, 1.6146789, 1.354515, 0.91784704, 0.4973684, 0.16494845,
            0.15949367, 0.7276996, 1.4171779, 1.8125, 1.7058824, 1.214876, 0.6142668,
        ];
        buffer.yin_cumulative_mean_normalized_difference();

        for (a, b) in buffer.iter().zip(expected.iter()) {
            assert!(cmp_f32(*a, *b))
        }
    }

    #[test]
    fn absolute_threshold_test() {
        let mut buffer = vec![
            0.0, 1.0, 1.4603175, 1.5461539, 1.6146789, 1.354515, 0.91784704, 0.4973684, 0.16494845,
            0.15949367, 0.7276996, 1.4171779, 1.8125, 1.7058824, 1.214876, 0.614266,
        ];
        let threshold = 0.2;
        let expected = Some(9);
        assert_eq!(buffer.yin_absolute_threshold(threshold), expected);
    }

    #[test]
    fn parabolic_interpolation_test() {
        let mut buffer = vec![
            0.0, 1.0, 1.4603175, 1.5461539, 1.6146789, 1.354515, 0.91784704, 0.4973684, 0.16494845,
            0.15949367, 0.7276996, 1.4171779, 1.8125, 1.7058824, 1.214876, 0.6142668,
        ];
        let tau_estimate = 9;
        let expected = 8.509509;

        assert!(cmp_f32(
            buffer.yin_parabolic_interpolation(tau_estimate),
            expected
        ));
    }

    #[test]
    fn yin_end_to_end() {
        let sample_rate = 44_100.0;
        let threshold = 0.20;

        let mut buffer = vec![
            0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0,
            -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0,
        ];
        let expected = (5181.9604, 0.8405063);
        let result = buffer.yin_pitch_detection(sample_rate, threshold);

        assert!(cmp_f32(result.0, expected.0));
        assert!(cmp_f32(result.1, expected.1));
    }
}
