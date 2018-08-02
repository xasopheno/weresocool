extern crate hound;
use instrument::oscillator::StereoWaveform;

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

pub fn write_composition_to_wav(composition_generator: fn() -> StereoWaveform) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let composition = composition_generator();
    let mut buffer = vec![0.0; composition.r_buffer.len() * 2];
    write_output_buffer(&mut buffer, composition);

    let mut writer = hound::WavWriter::create("composition.wav", spec).unwrap();
    for sample in buffer {
        writer.write_sample(sample).unwrap();
    }
}
