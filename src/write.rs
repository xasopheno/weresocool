use crate::generation::Op4D;
use crate::instrument::StereoWaveform;
use crate::settings::{default_settings, Settings};
use csv::Writer;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use weresocool_error::Error;

const SETTINGS: Settings = default_settings();

pub fn write_output_buffer(out_buffer: &mut [f32], stereo_waveform: StereoWaveform) {
    let mut l_idx = 0;
    let mut r_idx = 0;
    for (n, sample) in out_buffer.iter_mut().enumerate() {
        if n % 2 == 0 {
            *sample = stereo_waveform.l_buffer[l_idx] as f32;
            l_idx += 1
        } else {
            *sample = stereo_waveform.r_buffer[r_idx] as f32;
            r_idx += 1
        }
    }
}

pub fn filename_from_string(s: &str) -> &str {
    let split: Vec<&str> = s.split('.').collect();
    let filename: Vec<&str> = split[0].split('/').collect();
    filename[filename.len() - 1]
}

fn wav_to_mp3_in_renders(filename: &str) {
    let filename = filename_from_string(filename);
    let filename = format!("./renders/{}{}", filename, ".mp3".to_string());

    let child = Command::new("ffmpeg")
        .args(&[
            "-v",
            "panic",
            "-i",
            "composition.wav",
            "-codec:a",
            "libmp3lame",
            "-qscale:a",
            "2",
            "-y",
            &filename,
        ])
        .spawn();

    let ecode = child
        .expect("ffmpeg failed to encode mp3")
        .wait()
        .expect("failed to wait on child");

    assert!(ecode.success());
    println!("Successful mp3 encoding.");
}

pub fn write_composition_to_wav(
    composition: StereoWaveform,
    filename: &str,
    mp3: bool,
    normalize: bool,
) {
    let spec = hound::WavSpec {
        channels: SETTINGS.channels as u16,
        sample_rate: SETTINGS.sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut buffer = vec![0.0; composition.r_buffer.len() * 2];

    write_output_buffer(&mut buffer, composition);
    if normalize {
        normalize_waveform(&mut buffer);
    }

    let mut writer = hound::WavWriter::create("composition.wav", spec).unwrap();
    for sample in buffer {
        writer
            .write_sample(sample)
            .expect("Error writing wave file.");
    }
    println!("Successful wav encoding.");

    if mp3 {
        wav_to_mp3_in_renders(filename);
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

pub fn write_composition_to_json(serialized: &str, filename: &str) -> std::io::Result<()> {
    let filename = filename_from_string(filename);
    dbg!(filename);
    let mut file = File::create(format!(
        "./renders/{}{}",
        filename,
        ".socool.json".to_string()
    ))?;

    println!(
        "{}.json was written and has \
         1 render stream(s).\
         ",
        filename
    );

    file.write_all(serialized.as_bytes())?;

    Ok(())
}

pub fn write_composition_to_csv(ops: &mut Vec<Op4D>, filename: &str) -> Result<(), Error> {
    let filename = filename_from_string(filename);
    dbg!(filename);

    let filename = &format!("renders/{}{}", filename, ".socool.csv".to_string());
    let path = Path::new(filename);
    let mut writer = Writer::from_path(&path)?;
    for op in ops {
        writer
            .serialize(op.to_op_csv_1d())
            .expect("CSV writer error");
    }

    Ok(())
}
