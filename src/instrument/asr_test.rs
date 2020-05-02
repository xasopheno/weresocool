#[cfg(test)]
pub mod tests {
    use crate::{
        helpers::cmp_f64,
        instrument::asr::*,
    };
    fn short_gain_at_index(index: usize, silence_next: bool) -> f64 {
        let past_gain = 0.5;
        let current_gain = 1.0;
        let attack_length = 10;
        let decay_length = 10;
        let total_length = 30;

        calculate_short_gain(
            past_gain,
            current_gain,
            silence_next,
            index,
            attack_length,
            decay_length,
            total_length,
        )
    }
    fn long_gain_at_index(index: usize, silence_now: bool) -> f64 {
        let past_gain = 0.5;
        let current_gain = if silence_now { 0.0 } else { 1.0 };
        let attack_length = 10;
        let decay_length = 10;
        let total_length = 30;

        calculate_long_gain(
            past_gain,
            current_gain,
            silence_now,
            index,
            attack_length,
            decay_length,
            total_length,
        )
    }

    #[test]
    fn test_calculate_short_gain_sound_now() {
        assert!(cmp_f64(short_gain_at_index(0, false), 0.5));
        assert!(cmp_f64(short_gain_at_index(5, false), 0.75));
        assert!(cmp_f64(short_gain_at_index(10, false), 1.0));
        assert!(cmp_f64(short_gain_at_index(25, false), 1.0));
    }
    #[test]
    fn test_calculate_short_gain_silence_now() {
        assert!(cmp_f64(short_gain_at_index(25, true), 0.5));
    }

    #[test]
    fn test_calculate_long_gain_silence_now() {
        assert!(cmp_f64(long_gain_at_index(0, true), 0.5));
        assert!(cmp_f64(long_gain_at_index(5, true), 0.25));
        assert!(cmp_f64(long_gain_at_index(10, true), 0.0));
    }
    #[test]
    fn test_calculate_long_gain_sound_next() {
        assert!(cmp_f64(long_gain_at_index(0, false), 0.5));
        assert!(cmp_f64(long_gain_at_index(5, false), 0.75));
        assert!(cmp_f64(long_gain_at_index(10, false), 1.0));
        assert!(cmp_f64(long_gain_at_index(25, false), 1.0));
    }
}
