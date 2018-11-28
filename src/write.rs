extern crate hound;
use event::Event;
use instrument::stereo_waveform::StereoWaveform;
use serde_json;
use std::fs::File;
use std::io::prelude::*;

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

pub fn write_composition_to_wav(composition: StereoWaveform) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut buffer = vec![0.0; composition.r_buffer.len() * 2];

    write_output_buffer(&mut buffer, composition);
    normalize_waveform(&mut buffer);

    let mut writer = hound::WavWriter::create("composition.wav", spec).unwrap();
    for sample in buffer {
        writer.write_sample(sample).unwrap();
    }
}

pub fn normalize_waveform(buffer: &mut Vec<f32>) {
    let mut max = 0.0;
    for sample in buffer.iter() {
        if (*sample).abs() > max {
            max = *sample;
        }
    }

    let normalization_ratio = 1.0 / max * 0.85;

    for sample in buffer.iter_mut() {
        *sample *= normalization_ratio
    }

    println!("Normalized by {}", normalization_ratio);
}

pub fn write_composition_to_json(
    composition: Vec<Vec<Event>>,
    filename: &String,
) -> std::io::Result<()> {
    let serialized = serde_json::to_string(&composition).unwrap();
    let mut file = File::create(format!("{}{}", filename, ".json".to_string()))?;

    println!("{}.json was written and has {} event stream(s).", filename, composition.len());

    file.write_all(serialized.as_bytes())?;
    Ok(())
}
