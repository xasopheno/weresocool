use std;

pub fn generate_sinewave(
    freq: f32,
    phase: f32,
    buffer_size: usize,
    sample_rate: f32,
) -> (Vec<f32>, f32) {
    let tau: f32 = std::f32::consts::PI * 2.0;
    let factor: f32 = freq * tau / sample_rate;
    if freq < 10.0 || freq > 2500.0 {
        return (vec![0.0; buffer_size], 0.0);
    }
    let mut waveform: Vec<usize> = (0..buffer_size).collect();

    let waveform: Vec<f32> = waveform
        .iter_mut()
        .map(|sample| (((*sample as f32 * factor) + phase) % tau).sin())
        .collect();

    let mut new_phase = ( buffer_size as f32 * factor) + phase;
    new_phase %= tau;
    (waveform, new_phase)
}

#[allow(dead_code)]
fn sine_to_square(sample: f32) -> f32 {
    let result: f32;
    if sample < 0.0 {
        result = -1.0;
    } else if sample > 0.0 {
        result = 1.0;
    } else {
        result = 0.0;
    }
    result
}

pub mod tests {
    use sine::generate_sinewave;
    #[test]
    fn test_sine_generator() {
        let expected = vec![
            0.0, 0.06279052, 0.12533323, 0.18738133, 0.2486899, 0.309017, 0.36812457, 0.4257793,
            0.4817537, 0.53582686,
        ];
        let (result, _) = generate_sinewave(441.0, 0.0, 10, 44100.0);
        assert_eq!(result, expected);
    }
}
