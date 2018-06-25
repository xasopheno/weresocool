use oscillator::{R, Gain};
use std;

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
//    if gain < 5.0 || freq < 10.0 {
//        return (vec![0.0; buffer_size], vec![0.0; ratios.len()]);
//    }

    let mut waveform: Vec<usize> = (0..buffer_size).collect();

    let waveform: Vec<f32> = waveform
        .iter_mut()
        .map(|sample| {
            (generate_sample_of_compound_waveform(*sample as f32, factor, &ratios, &phases, tau))
        })
        .collect();

    let delta: f32 = (gain.current - gain.past)/ buffer_size;

    let gain_mask: Vec<usize> = (0..buffer_size).collect();
    let gain_mask: Vec<f32> = gain_mask
        .iter()
        .map(|index: usize| {*index as f32 * delta + gain.past})
        .collect();

    waveform.iter()
        .zip(gain_mask.iter())
        .map(|ref sample, ref gain| {
            sample * gain
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
            0.094077356,
            0.18713482,
            0.27816567,
            0.3661894,
            0.45026422,
            0.52949953,
            0.60306656,
            0.6702096,
            0.73025507,
        ];
        let (result, _) = generate_waveform(
            441.0,
            &vec![R::atio(2, 1), R::atio(3, 2), R::atio(1, 1)],
            &vec![0.0, 0.0, 0.0],
            10,
            44100.0,
        );
        assert_eq!(expected, result);
    }

}
