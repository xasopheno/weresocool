#[allow(dead_code)]
pub fn calculate_attack_gain(
    past_gain: f64,
    current_gain: f64,
    attack_index: usize,
    attack_length: usize,
) -> f64 {
    let distance = current_gain - past_gain;
    past_gain + (distance * attack_index as f64 / attack_length as f64)
}

#[allow(dead_code)]
pub fn calculate_decay_gain(current_gain: f64, decay_index: usize, decay_length: usize) -> f64 {
    let distance = -current_gain;
    current_gain + (distance * decay_index as f64 / decay_length as f64)
}

#[allow(dead_code)]
pub fn calculate_gain(
    past_gain: f64,
    current_gain: f64,
    silence_next: bool,
    index: usize,
    attack_length: usize,
    decay_length: usize,
    total_length: usize,
) -> f64 {
    // short gain
    let decay = true;
    if index < attack_length {
        calculate_attack_gain(past_gain, current_gain, index, attack_length)
    } else if decay {
        calculate_decay_gain(current_gain, total_length - index, attack_length)
    } else {
        current_gain
    }
}

#[test]
fn test_calculate_attack() {
    let past_gain = 0.5;
    let current_gain = 1.0;
    let silence_next = false;
    let attack_length = 10;
    let decay_length = 10;
    let total_length = 30;

    let gain = calculate_gain(
        past_gain,
        current_gain,
        silence_next,
        0,
        attack_length,
        decay_length,
        total_length,
    );
    assert_eq!(gain, 0.5);
    let gain = calculate_gain(
        past_gain,
        current_gain,
        silence_next,
        5,
        attack_length,
        decay_length,
        total_length,
    );
    assert_eq!(gain, 0.75);

    let gain = calculate_gain(
        past_gain,
        current_gain,
        silence_next,
        25,
        attack_length,
        decay_length,
        total_length,
    );
    assert_eq!(gain, 0.5);
}
