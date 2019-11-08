pub fn calculate_attack_gain(
    past_gain: f64,
    current_gain: f64,
    attack_index: usize,
    attack_length: usize,
) -> f64 {
    let distance = current_gain - past_gain;
    past_gain + (distance * attack_index as f64 / attack_length as f64)
}

pub fn calculate_decay_gain(current_gain: f64, decay_index: usize, decay_length: usize) -> f64 {
    let distance = -current_gain;
    current_gain + (distance * decay_index as f64 / decay_length as f64)
}
#[allow(dead_code)]
pub fn calculate_gain(
    past_gain: f64,
    current_gain: f64,
    silence_now: bool,
    silence_next: bool,
    index: usize,
    attack_length: usize,
    decay_length: usize,
    total_length: usize,
    decay_type: usize,
) -> f64 {
    if decay_type == 2 {
        calculate_long_gain(
            past_gain,
            current_gain,
            silence_now,
            silence_next,
            index,
            attack_length,
            decay_length,
            total_length,
        )
    } else {
        calculate_short_gain(
            past_gain,
            current_gain,
            silence_next,
            index,
            attack_length,
            decay_length,
            //1024,
            //1024,
            total_length,
        )
    }
}

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
        calculate_attack_gain(past_gain, current_gain, index, attack_length)
    } else if index > total_length - decay_length && silence_next {
        calculate_decay_gain(current_gain, total_length - index, attack_length)
    } else {
        current_gain
    }
}
#[allow(dead_code)]
pub fn calculate_long_gain(
    past_gain: f64,
    current_gain: f64,
    silence_now: bool,
    silence_next: bool,
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
        calculate_attack_gain(past_gain, current_gain, index, attack_length)
    } else if index < decay_length && silence_now {
        calculate_decay_gain(current_gain, index, decay_length)
    } else {
        current_gain
    }
}

pub fn is_short(total_length: usize, attack_length: usize, decay_length: usize) -> bool {
    total_length <= attack_length + decay_length
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
        11,
        attack_length,
        decay_length,
        total_length,
    );
    assert_eq!(gain, 1.0);

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
