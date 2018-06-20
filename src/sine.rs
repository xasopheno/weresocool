use std;
use oscillator::{R};


pub fn generate_waveform(
    freq: f32,
    ratios: &Vec<R>,
    phases: &Vec<f32>,
    buffer_size: usize,
    sample_rate: f32,
) -> (Vec<f32>, Vec<f32>) {
    let tau: f32 = std::f32::consts::PI * 2.0;
    let factor: f32 = freq * tau / sample_rate;
    if freq < 10.0 || freq > 2500.0 {
        return (vec![0.0; buffer_size], vec![0.0; ratios.len()]);
    }

    let mut waveform: Vec<usize> = (0..buffer_size).collect();

    let waveform: Vec<f32> = waveform
        .iter_mut()
        .map(|sample|
            (generate_sample_of_compound_waveform(*sample as f32, factor, &ratios, &phases, tau))
        )
        .collect();

    let new_phases = generate_phase_array(factor, &ratios, &phases, tau, buffer_size);

    (waveform, new_phases)
}

fn generate_sample_of_compound_waveform(sample: f32, factor: f32, ratios: &Vec<R>, phases: &Vec<f32>, tau: f32) -> f32 {
    let compound_sample: f32 = ratios.iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| (
            generate_sample_of_individual_waveform(sample, ratio.decimal, factor, **phase, tau))
        )
        .sum();
    let normalized_compound_sample = compound_sample / ratios.len() as f32;

    normalized_compound_sample
}

fn generate_sample_of_individual_waveform(sample: f32, ratio: f32, factor: f32, phase: f32, tau: f32) -> f32 {
    (((sample as f32 * factor * ratio) + phase) % tau).sin()
}

fn generate_phase_array(factor: f32, ratios:&Vec<R>, phases: &Vec<f32>, tau: f32, buffer_size: usize) -> Vec<f32> {
    ratios.iter()
        .zip(phases.iter())
        .map(| (ref ratio, ref phase)|
             calculate_phase(buffer_size as f32, factor, ratio.decimal, **phase, tau))
        .collect()
}

fn calculate_phase(buffer_size: f32, factor: f32, ratio: f32, phase: f32, tau: f32) -> f32 {
    ((buffer_size as f32 * factor * ratio) + phase) % tau
}

pub mod tests {
    use sine::generate_waveform;
    use oscillator::R;
    #[test]
    fn test_sine_generator() {
        let expected = vec![0.0, 0.33333334, 0.6666667, 1.0, 1.3333334, 1.6666666, 2.0, 2.3333333, 2.6666667, 3.0];
        let (result, _) = generate_waveform(
            441.0,
            &vec![R::atio(2, 1), R::atio(3, 2), R::atio(1, 1)],
            &vec![0.0, 0.0, 0.0],
            10,
            44100.0);
        assert_eq!(expected, result);
    }
}
