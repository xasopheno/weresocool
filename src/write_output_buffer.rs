use oscillator::{StereoWaveform};

pub fn write_output_buffer(out_buffer: &mut [f32], stereo_waveform: StereoWaveform) {
    let mut l_idx = 0;
    let mut r_idx = 0;
    for n in 0..out_buffer.len() {
        if n % 2 == 0 {
            out_buffer[n] = stereo_waveform.l_buffer[l_idx];
            l_idx += 1
        } else {
            out_buffer[n] = stereo_waveform.r_buffer[r_idx];
            r_idx += 1
        }
    }
}
