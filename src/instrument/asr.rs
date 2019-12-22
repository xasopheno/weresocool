use crate::instrument::voice::{GainInput, Voice};
use socool_ast::ASR;

impl Voice {
    pub fn calculate_gain(&mut self, gi: GainInput) -> f64 {
        if self.asr == ASR::Long {
            calculate_long_gain(
                gi.past_gain,
                gi.current_gain,
                gi.silence_now,
                gi.index,
                gi.attack_length,
                gi.decay_length,
                gi.total_samples,
            )
        } else {
            calculate_short_gain(
                gi.past_gain,
                gi.current_gain,
                gi.silent_next,
                gi.index,
                gi.attack_length,
                gi.decay_length,
                gi.total_samples,
            )
        }
    }
}

/// Calculate gain when decay happens during current op
pub fn calculate_short_gain(
    past_gain: f64,
    current_gain: f64,
    silence_next: bool,
    index: usize,
    mut attack_length: usize,
    mut decay_length: usize,
    total_length: usize,
) -> f64 {
    let short = is_short(total_length, attack_length, decay_length);
    if short {
        attack_length = total_length / 2;
        decay_length = total_length / 2;
    };

    if index < attack_length {
        gain_at_index(past_gain, current_gain, index, attack_length)
    } else if index > total_length - decay_length && silence_next {
        gain_at_index(current_gain, 0.0, total_length - index, decay_length)
    } else {
        current_gain
    }
}
/// Calculate gain when decay happens during next op
pub fn calculate_long_gain(
    past_gain: f64,
    current_gain: f64,
    silence_now: bool,
    index: usize,
    mut attack_length: usize,
    mut decay_length: usize,
    total_length: usize,
) -> f64 {
    let short = is_short(total_length, attack_length, decay_length);
    if short {
        attack_length = total_length;
        decay_length = total_length;
    };
    if index < attack_length {
        gain_at_index(past_gain, current_gain, index, attack_length)
    } else if index < decay_length && silence_now {
        gain_at_index(current_gain, 0.0, index, decay_length)
    } else {
        current_gain
    }
}
pub fn gain_at_index(start: f64, target: f64, index: usize, length: usize) -> f64 {
    let distance = target - start;
    start + (distance * index as f64 / length as f64)
}

pub fn is_short(total_length: usize, attack_length: usize, decay_length: usize) -> bool {
    total_length <= attack_length + decay_length
}
