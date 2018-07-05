use oscillator::{Gain, R};
use settings::Settings;
use std;

pub struct Generator {
    pub generate:
        fn(freq: f32, gain: &Gain, ratios: &Vec<R>, phases: &Vec<f32>, settings: &Settings)
            -> (Vec<f32>, Vec<f32>, f32),
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            generate: generate_waveform,
        }
    }
}

fn tau() -> f32 {
    std::f32::consts::PI * 2.0
}

pub fn generate_waveform(
    base_frequency: f32,
    gain: &Gain,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    settings: &Settings,
) -> (Vec<f32>, Vec<f32>, f32) {
    if base_frequency == 0.0 {
        return (
            vec![0.0; settings.buffer_size],
            vec![0.0; settings.buffer_size],
            1.0,
        );
    }
    let factor: f32 = tau() / settings.sample_rate;
    //        let base_frequency = base_frequency * 2.0;
    let mut waveform: Vec<usize> = (0..settings.buffer_size).collect();
    let gain_mask: Vec<f32> = generate_gain_mask(settings.buffer_size, gain);

    let normalization = loudness_normalization(base_frequency);
    //    println!("normalization {}, freq {}", normalization, base_frequency);

    let waveform: Vec<f32> = waveform
        .iter_mut()
        .zip(gain_mask.iter())
        .map(|(sample, gain_delta)| {
            generate_sample_of_compound_waveform(
                *sample as f32,
                base_frequency,
                factor,
                &ratios,
                &phases,
            ) * *gain_delta * normalization * 10.0
        })
        .collect();

    let new_phases = generate_phase_array(
        base_frequency,
        factor,
        &ratios,
        &phases,
        settings.buffer_size as usize,
    );

    (waveform, new_phases, normalization)
}

fn generate_gain_mask(buffer_size: usize, gain: &Gain) -> Vec<f32> {
    let mut gain_mask: Vec<usize> = (0..buffer_size).collect();

    let delta: f32 = (gain.current - gain.past) / buffer_size as f32;
    let gain_mask: Vec<f32> = gain_mask
        .iter_mut()
        .map(|index| *index as f32 * delta + gain.past)
        .collect();

    gain_mask
}

pub fn freq_to_sones(frequency: f32) -> f32 {
    // http://www.ukintpress-conferences.com/conf/08txeu_conf/pdf/day_1/01-06-garcia.pdf
    1.0 / 2.0_f32.powf(((20.0 * (frequency).log10()) - 40.0) / 10.0)
}

pub fn loudness_normalization(base_frequency: f32) -> f32 {
    let mut normalization = freq_to_sones(base_frequency);
    if normalization.is_nan() || normalization.is_infinite() || normalization > 1.0 {
        normalization = 1.0;
    };
    normalization
}

fn generate_sample_of_compound_waveform(
    sample: f32,
    base_frequency: f32,
    factor: f32,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
) -> f32 {
    let compound_sample: f32 = ratios
        .iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| {
            let frequency = (base_frequency * ratio.decimal) + ratio.offset;
            (generate_sample_of_individual_waveform(sample, frequency, factor, **phase, ratio.gain))
        })
        .sum();
    let normalized_compound_sample = compound_sample / ratios.len() as f32;

    normalized_compound_sample
}

fn generate_sample_of_individual_waveform(
    sample: f32,
    frequency: f32,
    factor: f32,
    phase: f32,
    gain: f32,
) -> f32 {
    let generated = (((sample as f32 * factor * frequency) + phase) % tau()).sin() * gain;
    generated
}

fn generate_phase_array(
    base_frequency: f32,
    factor: f32,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    buffer_size: usize,
) -> Vec<f32> {
    ratios
        .iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| {
            let frequency = base_frequency * ratio.decimal + ratio.offset;
            calculate_individual_phase(frequency, buffer_size as f32, factor, **phase)
        })
        .collect()
}

fn calculate_individual_phase(frequency: f32, buffer_size: f32, factor: f32, phase: f32) -> f32 {
    ((buffer_size as f32 * factor * frequency) + phase) % tau()
}

pub mod tests {
    use super::*;
    use settings::get_test_settings;
    #[test]
    fn test_sine_generator() {
        let expected = vec![
            0.0, 0.38888013, 0.78120327, 1.1726003, 1.5586493, 1.9349337, 2.2971044, 2.640939,
            2.9624, 3.2576942,
        ];
        let (result, _, _) = generate_waveform(
            441.0,
            &Gain::new(1.0, 1.1),
            &vec![
                R::atio(2, 1, 0.0, 1.0),
                R::atio(3, 2, 0.0, 1.0),
                R::atio(1, 1, 0.0, 1.0),
            ],
            &vec![0.0, 0.0, 0.0],
            &get_test_settings(),
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn test_loudness_normalization() {
        let expected = 0.20415603;
        let result = loudness_normalization(1400.0);
        assert_eq!(expected, result);

        let expected = 1.0;
        let result = loudness_normalization(40.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_calculate_individual_phase() {
        let expected = 0.037287712;
        let result =
            calculate_individual_phase(200.0, 512.0, std::f32::consts::PI * 2.0 * 44_100.0, 1.112);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_generate_gain_mask() {
        let expected = vec![
            0.8, 0.78000003, 0.76, 0.74, 0.72, 0.70000005, 0.68, 0.66, 0.64, 0.62,
        ];
        let result = generate_gain_mask(10, &Gain::new(0.8, 0.6));
        assert_eq!(expected, result);
    }

    #[test]
    fn test_tau() {
        let expected = 6.2831855;
        let result = tau();
        assert_eq!(expected, result);
    }
}
