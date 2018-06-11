use std;

pub fn generate_sinewave(sample_rate: f32, buffer_size: f32, freq: f32) -> Vec<f32> {
        let tau: f32 = std::f32::consts::PI * 2.0;
        let factor: f32 = freq * tau / sample_rate;
        let mut waveform: Vec<usize> = (0..buffer_size as usize).collect();

        let waveform: Vec<f32> = waveform.iter_mut()
            .map(|sample| (*sample as f32 * factor).sin())
            .collect();

        waveform
}
