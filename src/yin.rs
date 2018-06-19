pub trait YinBuffer {
    fn yin_pitch_detection(&mut self, sample_rate: f32, threshold: f32) -> f32;
    fn get_better_tau(&mut self, tau: usize, sample_rate: f32) -> f32;
    fn yin_difference(&mut self);
    fn yin_absolute_threshold(&mut self, threshold: f32) -> Option<usize>;
    fn yin_parabolic_interpolation(&mut self, tau_estimate: usize) -> f32;
    fn yin_cumulative_mean_normalized_difference(&mut self);
    fn gain(&mut self) -> f32;
}

impl YinBuffer for Vec<f32> {
    fn gain(&mut self) -> f32 {
        let max: f32 = self.iter().cloned().fold(0.0, |mut sum, x: f32 | {sum += x.powi(2); sum});;

        let gain = 20.0 * max.log10();
//        println!("{}", gain);
        gain
    }

    fn yin_pitch_detection(&mut self, sample_rate: f32, threshold: f32) -> f32 {
        self.yin_difference();
        self.yin_cumulative_mean_normalized_difference();
        let (probability, pitch_in_hertz) =
            if let Some(tau) = self.yin_absolute_threshold(threshold) {
                (1.0 - self[tau], self.get_better_tau(tau, sample_rate))
            } else {
                (-1.0, 0.0)
            };

        if probability > 0.5 && probability < 1.0 {
            pitch_in_hertz
        } else {
            0.0
        }
    }

    fn get_better_tau(&mut self, tau: usize, sample_rate: f32) -> f32 {
        let better_tau = self.yin_parabolic_interpolation(tau);
        let pitch_in_hertz = sample_rate / better_tau;
        pitch_in_hertz
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
        let buffer_size = self.len();
        let mut running_sum: f32 = 0.0;

        for tau in 1..buffer_size {
            running_sum += self[tau];
            self[tau] *= tau as f32 / running_sum;
        }
    }

    fn yin_absolute_threshold(&mut self, threshold: f32) -> Option<usize> {
        let mut iter = self.iter()
            .enumerate()
            .skip(2)
            .skip_while(|(_, &sample)| sample > threshold);
        let tripped_threshold = iter.next()?;

        let (_, mut previous_sample) = tripped_threshold;
        for (index, sample) in iter {
            if sample > previous_sample {
                return Some(index - 1);
            };
            previous_sample = sample;
        }

        Some(self.len() - 1)
    }

    fn yin_parabolic_interpolation(&mut self, tau_estimate: usize) -> f32 {
        let better_tau: f32;

        let x0: usize = if tau_estimate < 1 {
            tau_estimate
        } else {
            tau_estimate - 1
        };

        let x2: usize = if tau_estimate + 1 < self.len() {
            tau_estimate + 1
        } else {
            tau_estimate
        };

        if x0 == tau_estimate {
            better_tau = if self[tau_estimate] <= self[x2] {
                tau_estimate as f32
            } else {
                x2 as f32
            }
        } else if x2 == tau_estimate {
            better_tau = if self[tau_estimate] <= self[x0] {
                tau_estimate as f32
            } else {
                x0 as f32
            }
        } else {
            let s0: f32 = self[x0];
            let s1: f32 = self[tau_estimate];
            let s2: f32 = self[x2];

            better_tau = tau_estimate as f32 + (s2 - s0) / (2.0 * (2.0 * s1 - s2 - s0));
        }

        better_tau
    }
}

#[cfg(test)]
mod tests {
    use yin::*;
    #[test]
    fn gain_test() {
        let mut buffer = vec![
            0.0, 0.06279052, 0.12533323, 0.18738133, 0.2486899, 0.309017, 0.36812457, 0.4257793,
            0.4817537, 0.53582686,
        ];
        let gain = buffer.gain();
        let expected = -5.419511;
        assert_eq!(gain, expected);
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
        assert_eq!(buffer, expected);
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
        assert_eq!(buffer, expected);
    }

    #[test]
    fn absolute_threshold_test() {
        let mut buffer = vec![
            0.0, 1.0, 1.4603175, 1.5461539, 1.6146789, 1.354515, 0.91784704, 0.4973684, 0.16494845,
            0.15949367, 0.7276996, 1.4171779, 1.8125, 1.7058824, 1.214876, 0.614266,
        ];
        let threshold = 0.20;
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

        assert_eq!(buffer.yin_parabolic_interpolation(tau_estimate), expected);
    }

    #[test]
    fn yin_end_to_end() {
        let sample_rate = 44_100.00;
        let threshold = 0.20;

        let mut buffer = vec![
            0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0,
            -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0,
        ];
        let expected = 5182.4375;

        assert_eq!(buffer.yin_pitch_detection(sample_rate, threshold), expected);
    }
}
