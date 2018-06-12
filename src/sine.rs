use std;

pub fn generate_sinewave(sample_rate: f32, buffer_size: f32, freq: f32) -> Vec<f32> {
    let tau: f32 = std::f32::consts::PI * 2.0;
    let factor: f32 = freq * tau / sample_rate;
    let mut waveform: Vec<usize> = (0..buffer_size as usize).collect();

    let waveform: Vec<f32> = waveform
        .iter_mut()
        .map(|sample| (*sample as f32 * factor).sin())
        .collect();

    // println!(":{?}", waveform);

    waveform
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
        let result: Vec<f32> = generate_sinewave(44100.0, 10.0, 441.0);
        assert_eq!(result, expected);
    }
}
