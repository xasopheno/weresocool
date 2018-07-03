use oscillator::{Gain, R};
use std;

pub struct Generator {
    pub generate: fn(
        freq: f32,
        gain: &Gain,
        ratios: &Vec<R>,
        phases: &Vec<f32>,
        buffer_size: usize,
        sample_rate: f32,
    ) -> (Vec<f32>, Vec<f32>),
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            generate: generate_waveform,
        }
    }
}

pub fn freq_to_sones(frequency: f32) -> f32 {
    // http://www.ukintpress-conferences.com/conf/08txeu_conf/pdf/day_1/01-06-garcia.pdf
    1.0 / 2.0_f32.powf(((10.0 * (frequency).log10()) - 40.0) / 10.0)
}

pub fn generate_waveform(
    base_frequency: f32,
    gain: &Gain,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    buffer_size: usize,
    sample_rate: f32,
) -> (Vec<f32>, Vec<f32>) {
    let tau: f32 = std::f32::consts::PI * 2.0;
    let factor: f32 = tau / sample_rate;
    //    let base_frequency = base_frequency * 2.0;
    let mut waveform: Vec<usize> = (0..buffer_size).collect();
    let mut gain_mask: Vec<usize> = (0..buffer_size).collect();

    let delta: f32 = (gain.current - gain.past) / buffer_size as f32;
    let mut gain_mask: Vec<f32> = gain_mask
        .iter_mut()
        .map(|index| *index as f32 * delta + gain.past)
        .collect();

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
                tau,
            ) * *gain_delta * 10.0
        })
        .collect();

    let new_phases =
        generate_phase_array(base_frequency, factor, &ratios, &phases, tau, buffer_size);

    (waveform, new_phases)
}

fn generate_sample_of_compound_waveform(
    sample: f32,
    base_frequency: f32,
    factor: f32,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    tau: f32,
) -> f32 {
    let compound_sample: f32 = ratios
        .iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| {
            let frequency = (base_frequency * ratio.decimal) + ratio.offset;
            (generate_sample_of_individual_waveform(
                sample, frequency, factor, **phase, ratio.gain, tau,
            ))
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
    tau: f32,
) -> f32 {
    let normalization =  1.0; //freq_to_sones(frequency);
    let generated = (((sample as f32 * factor * frequency) + phase) % tau).sin()
        * gain
        * normalization;
    generated
}

fn generate_phase_array(
    base_frequency: f32,
    factor: f32,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    tau: f32,
    buffer_size: usize,
) -> Vec<f32> {
    ratios
        .iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| {
            let frequency = base_frequency * ratio.decimal + ratio.offset;
            calculate_individual_phase(frequency, buffer_size as f32, factor, **phase, tau)
        })
        .collect()
}

fn calculate_individual_phase(
    frequency: f32,
    buffer_size: f32,
    factor: f32,
    phase: f32,
    tau: f32,
) -> f32 {
    ((buffer_size as f32 * factor * frequency) + phase) % tau
}

pub mod tests {
    use super::*;
    use oscillator::R;
    #[test]
    fn test_sine_generator() {
        let expected = vec![
            0.0,
            0.095018126,
            0.19087751,
            0.28651062,
            0.38083696,
            0.4727775,
            0.56126946,
            0.6452813,
            0.7238264,
            0.7959779,
        ];
        let (result, _) = generate_waveform(
            441.0,
            &Gain::new(1.0, 1.1),
            &vec![
                R::atio(2, 1, 0.0, 1.0),
                R::atio(3, 2, 0.0, 1.0),
                R::atio(1, 1, 0.0, 1.0),
            ],
            &vec![0.0, 0.0, 0.0],
            10,
            44100.0,
        );
        assert_eq!(expected, result);
    }

}
