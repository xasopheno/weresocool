use crate::{gain::gain_at_index, voice::Voice};
use weresocool_ast::ASR;

impl Voice {
    pub fn calculate_op_gain(
        &mut self,
        next_out: bool,
        silence_now: bool,
        silence_next: bool,
        index: usize,
        total_length: usize,
    ) -> f64 {
        if next_out || self.asr == ASR::Long {
            calculate_long_gain(
                self.past.gain,
                self.current.gain,
                self.sustain,
                self.release,
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
                self.sustain,
                self.release,
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
    sustain_level: f64,
    release_length: usize,
    silence_next: bool,
    index: usize,
    mut attack_length: usize,
    mut decay_length: usize,
    total_length: usize,
) -> f64 {
    // Scale A/D/(optional R) to fit within total_length when needed
    let mut release_len = if silence_next { release_length } else { 0 };
    let needed = attack_length + decay_length + release_len;
    if needed > total_length && needed > 0 {
        let scale = total_length as f64 / needed as f64;
        attack_length = ((attack_length as f64 * scale).max(1.0)).round() as usize;
        decay_length = ((decay_length as f64 * scale).max(1.0)).round() as usize;
        release_len = ((release_len as f64 * scale).max(0.0)).round() as usize;
    }

    // Phase boundaries
    let sustain_gain = current_gain * sustain_level;
    let attack_end = attack_length;
    let decay_end = attack_length + decay_length;
    let release_start = if silence_next {
        total_length.saturating_sub(release_len)
    } else {
        usize::MAX // no release inside this op
    };

    if index < attack_end {
        gain_at_index(past_gain, current_gain, index, attack_length)
    } else if index < decay_end {
        let local_index = index - attack_end;
        gain_at_index(current_gain, sustain_gain, local_index, decay_length)
    } else if index >= release_start {
        let time_into_release = index - release_start;
        gain_at_index(sustain_gain, 0.0, time_into_release, release_len)
    } else {
        sustain_gain
    }
}
/// Calculate gain when decay happens during next op
pub fn calculate_long_gain(
    past_gain: f64,
    current_gain: f64,
    sustain_level: f64,
    release_length: usize,
    silence_now: bool,
    index: usize,
    mut attack_length: usize,
    mut decay_length: usize,
    total_length: usize,
) -> f64 {
    if silence_now {
        // Silent op: perform release only, starting immediately, from previous op's sustain level
        let base_gain = past_gain;
        let sustain_gain = base_gain * sustain_level;
        if index < release_length {
            gain_at_index(sustain_gain, 0.0, index, release_length)
        } else {
            0.0
        }
    } else {
        // Sounding op: perform A/D then hold sustain; release handled in following silent op
        let short = is_short(total_length, attack_length, decay_length);
        if short {
            attack_length = total_length;
            decay_length = total_length;
        };
        let sustain_gain = current_gain * sustain_level;

        if index < attack_length {
            gain_at_index(past_gain, current_gain, index, attack_length)
        } else if index < attack_length + decay_length {
            let local_index = index - attack_length;
            gain_at_index(current_gain, sustain_gain, local_index, decay_length)
        } else {
            sustain_gain
        }
    }
}

pub const fn is_short(total_length: usize, attack_length: usize, decay_length: usize) -> bool {
    total_length <= attack_length + decay_length
}
