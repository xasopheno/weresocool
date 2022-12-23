use crate::generation::Op4D;
use csv::Writer;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, Cursor};
use std::path::PathBuf;
use weresocool_error::Error;
use weresocool_instrument::{Normalize, StereoWaveform};
#[cfg(not(any(target_os = "windows", feature = "wasm")))]
use weresocool_lame::Lame;
use weresocool_shared::{get_settings, Settings};

const SETTINGS: Settings = get_settings();

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

pub fn new_write_output_buffer(
    out_buffer: &mut [f32],
    stereo_waveform: StereoWaveform,
    offset: Vec<f32>,
) {
    let mut l_idx = 0;
    let mut r_idx = 0;
    for (n, sample) in out_buffer.iter_mut().enumerate() {
        if n % 2 == 0 {
            *sample = offset[n] * stereo_waveform.l_buffer[l_idx] as f32;
            l_idx += 1
        } else {
            *sample = offset[n] * stereo_waveform.r_buffer[r_idx] as f32;
            r_idx += 1
        }
    }
}

pub fn filename_from_string(s: &str) -> &str {
    let split: Vec<&str> = s.split('.').collect();
    let filename: Vec<&str> = split[0].split('/').collect();
    filename[filename.len() - 1]
}

#[cfg(any(feature = "wasm", target_os = "windows"))]
pub fn write_composition_to_mp3(_composition: StereoWaveform) -> Result<Vec<u8>, Error> {
    Err(Error::with_msg("Mp3 not available on this platform"))
}

#[cfg(all(feature = "app", not(target_os = "windows")))]
pub fn write_composition_to_mp3(mut composition: StereoWaveform) -> Result<Vec<u8>, Error> {
    composition.normalize();

    let l_buffer = composition.l_buffer;
    let r_buffer = composition.r_buffer;
    let length: f32 = l_buffer.len() as f32 * (0.37);
    let mp3buf = &mut vec![0_u8; length.ceil() as usize];

    let mut l = Lame::new().ok_or(weresocool_lame::Error::InternalError)?;

    l.init_params()?;
    l.encode_f32(l_buffer.as_slice(), r_buffer.as_slice(), mp3buf)?;

    Ok(mp3buf.to_vec())
}

#[test]
fn write_composition_to_mp3_test() {
    let sw = StereoWaveform::new_with_buffer(vec![0.0; 2048]);
    let mp3 = write_composition_to_mp3(sw);
    assert_eq!(mp3.unwrap().len(), 758)
}

pub fn write_composition_to_wav(mut composition: StereoWaveform) -> Result<Vec<u8>, Error> {
    composition.normalize();

    let spec = hound::WavSpec {
        channels: SETTINGS.channels as u16,
        sample_rate: SETTINGS.sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let c = Cursor::new(Vec::new());

    let mut buf_writer = BufWriter::new(c);
    let mut writer = hound::WavWriter::new(&mut buf_writer, spec)?;
    let mut buffer = vec![0.0; composition.r_buffer.len() * 2];
    normalize_waveform(&mut buffer);
    write_output_buffer(&mut buffer, composition);
    for sample in &buffer {
        writer
            .write_sample(*sample)
            .expect("Error writing wave file.");
    }
    writer.flush()?;
    writer.finalize()?;

    Ok(buf_writer.into_inner().unwrap().into_inner())
}

#[test]
fn write_composition_to_wav_test() {
    let sw = StereoWaveform::new_with_buffer(vec![0.0; 10]);
    let wav = write_composition_to_wav(sw);
    assert_eq!(wav.unwrap().len(), 148)
}

pub fn normalize_waveform(buffer: &mut [f32]) {
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
    serialized: &str,
    filename: &str,
    mut output_dir: PathBuf,
) -> std::io::Result<()> {
    let filename = filename_from_string(filename);
    let filename = &format!("{}.socool.json", filename);
    output_dir.push(filename);
    let mut file = File::create(output_dir)?;

    println!(
        "{}.json was written and has \
         1 render stream(s).\
         ",
        filename
    );

    file.write_all(serialized.as_bytes())?;

    Ok(())
}

pub fn write_composition_to_csv(
    ops: &mut Vec<Op4D>,
    filename: &str,
    mut output_dir: PathBuf,
) -> Result<(), Error> {
    let filename = filename_from_string(filename);
    let filename = &format!("{}.socool.csv", filename);
    output_dir.push(filename);
    let mut writer = Writer::from_path(output_dir.as_path())?;
    for op in ops {
        writer.serialize(op.to_op_csv()).expect("CSV writer error");
    }

    Ok(())
}
