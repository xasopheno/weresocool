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

pub fn generate_waveform(
    freq: f32,
    gain: &Gain,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    buffer_size: usize,
    sample_rate: f32,
) -> (Vec<f32>, Vec<f32>) {
    let tau: f32 = std::f32::consts::PI * 2.0;
    let factor: f32 = freq * tau / sample_rate;

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
            generate_sample_of_compound_waveform(*sample as f32, factor, &ratios, &phases, tau)
                * *gain_delta
        })
        .collect();

    let new_phases = generate_phase_array(factor, &ratios, &phases, tau, buffer_size);

    (waveform, new_phases)
}

fn generate_sample_of_compound_waveform(
    sample: f32,
    factor: f32,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    tau: f32,
) -> f32 {
    let compound_sample: f32 = ratios
        .iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| {
            (generate_sample_of_individual_waveform(sample, ratio.decimal, factor, **phase, tau))
        })
        .sum();
    let normalized_compound_sample = compound_sample / ratios.len() as f32;

    normalized_compound_sample
}

fn generate_sample_of_individual_waveform(
    sample: f32,
    ratio: f32,
    factor: f32,
    phase: f32,
    tau: f32,
) -> f32 {
    (((sample as f32 * factor * ratio) + phase) % tau).sin()
}

fn generate_phase_array(
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
            calculate_individual_phase(buffer_size as f32, factor, ratio.decimal, **phase, tau)
        })
        .collect()
}

fn calculate_individual_phase(
    buffer_size: f32,
    factor: f32,
    ratio: f32,
    phase: f32,
    tau: f32,
) -> f32 {
    ((buffer_size as f32 * factor * ratio) + phase) % tau
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
            0.4727774,
            0.5612695,
            0.64528126,
            0.7238264,
            0.79597807,
        ];
        let (result, _) = generate_waveform(
            441.0,
            &Gain::new(1.0, 1.1),
            &vec![R::atio(2, 1), R::atio(3, 2), R::atio(1, 1)],
            &vec![0.0, 0.0, 0.0],
            10,
            44100.0,
        );
        assert_eq!(expected, result);
    }

}
