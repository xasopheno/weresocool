use oscillator::Gain;
use ratios::R;
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
    let factor: f32 = tau() / settings.sample_rate;
    let base_frequency = base_frequency * 2.0;
    let mut waveform: Vec<usize> = (0..settings.buffer_size).collect();
    let loudness = loudness_normalization(base_frequency);

    //    println!("{:?}, {:?}, {:?}", base_frequency, gain, phases);

    let gain_mask: Vec<f32> = generate_gain_mask(settings.buffer_size, gain, loudness);

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
            ) * *gain_delta * settings.gain_multiplier
        })
        .collect();

    let new_phases = generate_phase_array(
        base_frequency,
        factor,
        &ratios,
        &phases,
        gain.current,
        settings.buffer_size as usize,
    );

    (waveform, new_phases, loudness)
}

fn generate_gain_mask(buffer_size: usize, gain: &Gain, loudness: f32) -> Vec<f32> {
    let mut gain_mask: Vec<usize> = (0..buffer_size).collect();
    let mut current_volume = gain.current * loudness;
    if current_volume > 1.0 {
        current_volume = 1.0
    }

    //    println!("{}, {}", gain.past, current_volume);

    let delta: f32 = (current_volume - gain.past) / (buffer_size as f32 - 1.0);
    let mut gain_mask: Vec<f32> = gain_mask
        .iter_mut()
        .map(|index| *index as f32 * delta + gain.past)
        .collect();

    gain_mask[buffer_size - 1] = current_volume;

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
    current_gain: f32,
    buffer_size: usize,
) -> Vec<f32> {
    if current_gain == 0.0 {
        return vec![0.0; ratios.len()];
    }

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

#[cfg(test)]
pub mod tests {
    use super::*;
    use settings::get_test_settings;
    #[test]
    fn test_sine_generator() {
        let expected = vec![
            0.0,
            2.6153219,
            3.726057,
            3.1853983,
            1.662159,
            0.12425217,
            -0.74386966,
            -0.83147734,
            -0.4828528,
            -0.13460012,
        ];
        let (result, _, _) = generate_waveform(
            1441.0,
            &Gain::new(0.5, 1.0),
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
            0.8, 0.7111111, 0.62222224, 0.5333333, 0.44444445, 0.35555556, 0.26666665, 0.17777777,
            0.08888888, 0.0,
        ];
        let result = generate_gain_mask(10, &Gain::new(0.8, 0.0), 1.0);
        assert_eq!(expected, result);

        let expected = vec![
            0.5, 0.5222222, 0.54444444, 0.56666666, 0.5888889, 0.6111111, 0.6333333, 0.65555555,
            0.67777777, 0.7,
        ];
        let result = generate_gain_mask(10, &Gain::new(0.5, 0.7), 1.0);
        assert_eq!(expected, result);

        let expected = vec![
            1.0, 0.95555556, 0.9111111, 0.8666667, 0.82222223, 0.7777778, 0.73333335, 0.6888889,
            0.64444447, 0.6,
        ];
        let result = generate_gain_mask(10, &Gain::new(1.0, 0.6), 1.0);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_tau() {
        let expected = 6.2831855;
        let result = tau();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_generate_sample_of_individual_waveform() {
        let result = 0.4731935;
        let expected =
            generate_sample_of_individual_waveform(0.12, 100.0, tau() * 44_100.0, 0.4, 1.0);
        assert_eq!(result, expected);
    }
}
