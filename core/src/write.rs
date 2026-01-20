use crate::generation::Op4D;
use csv::Writer;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, Cursor};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use weresocool_error::Error;
use weresocool_instrument::{Normalize, StereoWaveform};
#[cfg(not(any(target_os = "windows", feature = "wasm")))]
use weresocool_lame::Lame;
use weresocool_shared::{Settings, timing_print};

// Diagnostic: track samples for discontinuity detection
static TOTAL_SAMPLES: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
static LAST_L: Lazy<AtomicI64> = Lazy::new(|| AtomicI64::new(0));
static LAST_R: Lazy<AtomicI64> = Lazy::new(|| AtomicI64::new(0));

fn f32_to_i64(f: f32) -> i64 { (f * 1_000_000.0) as i64 }
fn i64_to_f32(i: i64) -> f32 { i as f32 / 1_000_000.0 }

pub fn write_output_buffer(out_buffer: &mut [f32], stereo_waveform: StereoWaveform) {
    let len = stereo_waveform.l_buffer.len();

    for i in 0..len {
        out_buffer[i * 2] = stereo_waveform.l_buffer[i] as f32;
        out_buffer[i * 2 + 1] = stereo_waveform.r_buffer[i] as f32;
    }
}

pub fn new_write_output_buffer(
    out_buffer: &mut [f32],
    stereo_waveform: StereoWaveform,
    offset: Vec<f32>,
) {
    let len = stereo_waveform.l_buffer.len();
    let click_detection = Settings::global().click_detection;

    if click_detection {
        // Track samples for discontinuity detection
        let start_sample = TOTAL_SAMPLES.fetch_add(len as u64, Ordering::Relaxed);
        let mut prev_l = i64_to_f32(LAST_L.load(Ordering::Relaxed));
        let mut prev_r = i64_to_f32(LAST_R.load(Ordering::Relaxed));

        for i in 0..len {
            let l = offset[i * 2] * stereo_waveform.l_buffer[i] as f32;
            let r = offset[i * 2 + 1] * stereo_waveform.r_buffer[i] as f32;

            // Detect discontinuity
            let delta_l = l - prev_l;
            let delta_r = r - prev_r;
            if delta_l.abs() > 0.2 || delta_r.abs() > 0.2 {
                let sample = start_sample + i as u64;
                let time_sec = sample as f64 / 44100.0;
                timing_print!("CLICK: sample={} t={:.3}s i={} L[{:.3}->{:.3}]d={:.3} R[{:.3}->{:.3}]d={:.3}",
                    sample, time_sec, i, prev_l, l, delta_l, prev_r, r, delta_r);
            }

            out_buffer[i * 2] = l;
            out_buffer[i * 2 + 1] = r;
            prev_l = l;
            prev_r = r;
        }

        if len > 0 {
            LAST_L.store(f32_to_i64(prev_l), Ordering::Relaxed);
            LAST_R.store(f32_to_i64(prev_r), Ordering::Relaxed);
        }
    } else {
        // Fast path: no click detection
        for i in 0..len {
            out_buffer[i * 2] = offset[i * 2] * stereo_waveform.l_buffer[i] as f32;
            out_buffer[i * 2 + 1] = offset[i * 2 + 1] * stereo_waveform.r_buffer[i] as f32;
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

#[cfg(all(feature = "app", not(target_os = "windows"), not(feature = "wasm")))]
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
        channels: Settings::global().channels as u16,
        sample_rate: Settings::global().sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let c = Cursor::new(Vec::new());

    let mut buf_writer = BufWriter::new(c);
    let mut writer = hound::WavWriter::new(&mut buf_writer, spec)?;
    let mut buffer = vec![0.0; composition.r_buffer.len() * 2];
    normalize_waveform(&mut buffer);
    write_output_buffer(&mut buffer, composition);
    // Optionally normalize here if desired; currently writing as-is after prior normalization
    for sample in &buffer {
        writer
            .write_sample(*sample)
            .map_err(|e| Error::with_msg(format!("Error writing WAV sample: {}", e)))?;
    }
    writer.flush()?;
    writer.finalize()?;

    Ok(buf_writer
        .into_inner()
        .map_err(|e| Error::with_msg(format!("Error finalizing WAV buffer: {}", e)))?
        .into_inner())
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
}

pub fn write_composition_to_json(
    serialized: &str,
    filename: &str,
    mut output_dir: PathBuf,
) -> std::io::Result<()> {
    let filename = filename_from_string(filename);
    let filename = &format!("{}.socool.data.json", filename);
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
