use crate::instrument::{gain::gain_at_index, voice::Voice};
use weresocool_ast::ASR;

impl Voice {
    pub fn calculate_op_gain(
        &mut self,
        silence_now: bool,
        silence_next: bool,
        index: usize,
        total_length: usize,
    ) -> f64 {
        if self.asr == ASR::Long {
            calculate_long_gain(
                self.past.gain,
                self.current.gain,
                silence_now,
                index,
                self.attack,
                self.decay,
                total_length,
            )
        } else {
            calculate_short_gain(
                self.past.gain,
                self.current.gain,
                silence_next,
                index,
                self.attack,
                self.decay,
                total_length,
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

pub const fn is_short(total_length: usize, attack_length: usize, decay_length: usize) -> bool {
    total_length <= attack_length + decay_length
}
