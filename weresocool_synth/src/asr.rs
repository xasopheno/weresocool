//! The gain envelope, as a pure function of position WITHIN THE NOTE.
//!
//! Nothing here may read a buffer length. These functions used to be reached
//! through `Voice::calculate_op_gain`, called once per buffer with the index
//! at the buffer's end — which made the envelope a block-rate staircase and,
//! on the unchunked path, a single sample of itself. They are now called from
//! inside the per-sample loop in `voice.rs` with `op.sample_index() + i`, the
//! same note-scoped clock the drum oscillators have always used.
//!
//! `past_gain` is supplied by the caller in RENDERED gain space — the value
//! the voice was actually emitting when this note began. Drums pass
//! `past_gain == current_gain` on a note-on so the attack branch collapses to
//! a no-op (their oscillators shape their own attack, and an outer ramp on top
//! buries the transient), but NOT when fading into silence: there this ramp is
//! the tail's fade-out, and flattening it would cut the ring dead.

use crate::gain::gain_at_index;

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
