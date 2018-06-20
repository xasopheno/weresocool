use std;
use oscillator::{R};


pub fn generate_sinewave(
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
            (apply_ratios(*sample as f32, factor, &ratios, &phases, tau))
        )
        .collect();

    let new_phases = calc_phases(factor, &ratios, &phases, tau, buffer_size);

    (waveform, new_phases)
}

fn calc_sample(sample: f32, ratio: f32, factor: f32, phase: f32, tau: f32) -> f32 {
    let result = (((sample as f32 * factor * ratio) + phase) % tau).sin();
    result

}

fn apply_ratios(sample: f32, factor: f32, ratios: &Vec<R>, phases: &Vec<f32>, tau: f32) -> f32 {
    let mut result: f32 = ratios.iter()
        .zip(phases.iter())
        .map(|(ref ratio, ref phase)| (
            calc_sample(sample, ratio.decimal, factor, **phase, tau))
        )
        .sum();
    result / ratios.len() as f32
}

fn calc_phases(factor: f32, ratios:&Vec<R>, phases: &Vec<f32>, tau: f32, buffer_size: usize) -> Vec<f32> {
    let ratios = ratios.iter()
        .zip(phases.iter())
        .map(| (ref ratio, ref phase)|
             calc_phase(buffer_size as f32, factor, ratio.decimal,  **phase, tau))
        .collect();
    ratios
}

fn calc_phase(buffer_size: f32, factor: f32, ratio: f32, phase: f32, tau: f32) -> f32 {
    ((buffer_size as f32 * factor * ratio) + phase) % tau
}

pub mod tests {
    use sine::generate_sinewave;
    #[test]
    fn test_sine_generator() {
        let expected = vec![
            0.0, 0.06279052, 0.12533323, 0.18738133, 0.2486899, 0.309017, 0.36812457, 0.4257793,
            0.4817537, 0.53582686,
        ];
        let (result, _) = generate_sinewave(441.0, (0.0, 0.0, 0.0), 10, 44100.0);
        assert_eq!(result, expected);
    }
}
