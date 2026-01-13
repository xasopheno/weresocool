//! FromSound: Analyze WAV files and convert to WereSoCool NormalForm
//!
//! This module provides the `from_sound_to_normalform` function that:
//! 1. Analyzes a WAV file using spectral analysis
//! 2. Converts the resulting tracks to PointOps with loudness pre-compensation
//! 3. Caches results to avoid re-analysis on every render

use crate::{NormalForm, PointOp, OscType, ASR};
use num_rational::Rational64;
use std::path::Path;
use std::fs;
use std::time::SystemTime;
use weresocool_error::Error;
use weresocool_from_sound::{
    Analyzer, AnalysisConfig, make_analyzer,
    read_wav_mono,
    VoiceAllocator, VoiceAllocConfig,
    TrackOut, AnalysisOutput,
};

/// Cache entry for FromSound analysis results
#[derive(serde::Serialize, serde::Deserialize)]
struct FromSoundCache {
    audio_path: String,
    audio_mtime: u64,
    voices: usize,
    operations: Vec<Vec<SerializedPointOp>>,
    length_ratio_num: i64,
    length_ratio_denom: i64,
}

/// Simplified PointOp for serialization (since PointOp doesn't implement Serialize)
#[derive(serde::Serialize, serde::Deserialize)]
struct SerializedPointOp {
    fm_num: i64,
    fm_denom: i64,
    fa_num: i64,
    fa_denom: i64,
    g_num: i64,
    g_denom: i64,
    l_num: i64,
    l_denom: i64,
    pm_num: i64,
    pm_denom: i64,
    pa_num: i64,
    pa_denom: i64,
    attack_num: i64,
    attack_denom: i64,
    decay_num: i64,
    decay_denom: i64,
    portamento_num: i64,
    portamento_denom: i64,
}

impl From<&PointOp> for SerializedPointOp {
    fn from(op: &PointOp) -> Self {
        Self {
            fm_num: *op.fm.numer(),
            fm_denom: *op.fm.denom(),
            fa_num: *op.fa.numer(),
            fa_denom: *op.fa.denom(),
            g_num: *op.g.numer(),
            g_denom: *op.g.denom(),
            l_num: *op.l.numer(),
            l_denom: *op.l.denom(),
            pm_num: *op.pm.numer(),
            pm_denom: *op.pm.denom(),
            pa_num: *op.pa.numer(),
            pa_denom: *op.pa.denom(),
            attack_num: *op.attack.numer(),
            attack_denom: *op.attack.denom(),
            decay_num: *op.decay.numer(),
            decay_denom: *op.decay.denom(),
            portamento_num: *op.portamento.numer(),
            portamento_denom: *op.portamento.denom(),
        }
    }
}

impl SerializedPointOp {
    fn to_point_op(&self) -> PointOp {
        PointOp {
            fm: Rational64::new(self.fm_num, self.fm_denom),
            fa: Rational64::new(self.fa_num, self.fa_denom),
            g: Rational64::new(self.g_num, self.g_denom),
            l: Rational64::new(self.l_num, self.l_denom),
            pm: Rational64::new(self.pm_num, self.pm_denom),
            pa: Rational64::new(self.pa_num, self.pa_denom),
            attack: Rational64::new(self.attack_num, self.attack_denom),
            decay: Rational64::new(self.decay_num, self.decay_denom),
            portamento: Rational64::new(self.portamento_num, self.portamento_denom),
            asr: ASR::Long,
            osc_type: OscType::Sine { pow: None },
            ..Default::default()
        }
    }
}

/// Main entry point: analyze audio file and return NormalForm
pub fn from_sound_to_normalform(path: &str, voices: usize, fps: usize) -> Result<NormalForm, Error> {
    let total_start = std::time::Instant::now();

    // Validate file exists
    if !Path::new(path).exists() {
        return Err(Error::with_msg(format!(
            "FromSound: audio file not found: '{}'\nMake sure the path is relative to the .socool file location.",
            path
        )));
    }

    // Try to load from cache first
    let cache_start = std::time::Instant::now();
    if let Some(nf) = load_from_cache(path, voices, fps)? {
        eprintln!("[FromSound] Cache load: {:?}, total: {:?}", cache_start.elapsed(), total_start.elapsed());
        return Ok(nf);
    }
    eprintln!("[FromSound] Cache miss check: {:?}", cache_start.elapsed());

    // Analyze the audio file
    let read_start = std::time::Instant::now();
    let (sample_rate, samples) = read_wav_mono(path)
        .map_err(|e| Error::with_msg(format!("Failed to read WAV file: {}", e)))?;
    eprintln!("[FromSound] WAV read: {:?}", read_start.elapsed());

    let config = AnalysisConfig {
        use_multi_res: true,
        use_esprit1: true,
        use_esprit_multi: false,
        use_harmonic: false,
        use_lpc_noise: false,
        use_reassignment: true,
        use_adaptive_window: true,
        use_phase_locking: true,
        use_mq_tracking: true,
    };

    let analysis_start = std::time::Instant::now();
    let analyzer = make_analyzer(
        sample_rate,
        2048, 512,      // FFT params
        8192, 1024,     // High-res FFT params
        120,            // max_peaks
        -70.0,          // min_db
        35.0,           // max_dev_hz
        2,              // max_gap
        3,              // min_len
        40, 30.0,       // ESPRIT1 params
        0, 40.0,        // ESPRIT multi params (unused)
        20,             // LPC order (unused)
        40.0, 1200.0, 12, 24, 0.03, 5.0, 0.04, 3,  // Harmonic params (unused)
        config,
    );

    let analysis = analyzer.run(&samples)
        .map_err(|e| Error::with_msg(format!("Analysis failed: {}", e)))?;
    eprintln!("[FromSound] Analysis: {:?}", analysis_start.elapsed());

    // Voice allocation
    let alloc_start = std::time::Instant::now();
    let selected_tracks = if voices > 0 && analysis.tracks.len() > voices {
        let alloc_config = VoiceAllocConfig {
            frame_duration: 1.0,
            frame_overlap: 0.5,
            max_voices: voices,
            continuity_weight: 2.0,
            max_freq_jump: 200.0,
        };
        let mut allocator = VoiceAllocator::new(alloc_config);
        allocator.allocate_voices(analysis.tracks.clone(), analysis.duration_sec)
    } else {
        analysis.tracks.clone()
    };
    eprintln!("[FromSound] Voice allocation: {:?}", alloc_start.elapsed());

    // Convert to NormalForm
    let convert_start = std::time::Instant::now();
    let nf = tracks_to_normalform(&selected_tracks, analysis.duration_sec, fps)?;
    eprintln!("[FromSound] Convert to NormalForm: {:?}", convert_start.elapsed());

    // Save to cache
    let save_start = std::time::Instant::now();
    save_to_cache(path, voices, fps, &nf)?;
    eprintln!("[FromSound] Save cache: {:?}", save_start.elapsed());

    eprintln!("[FromSound] Total (no cache): {:?}", total_start.elapsed());
    Ok(nf)
}

/// Convert analysis tracks to WereSoCool NormalForm
fn tracks_to_normalform(tracks: &[TrackOut], duration_sec: f32, fps: usize) -> Result<NormalForm, Error> {
    let base_freq = 440.0_f32;

    let mut voices: Vec<Vec<PointOp>> = Vec::new();
    let mut voice_end_times: Vec<f32> = Vec::new();

    // Sort tracks by start time
    let mut track_infos: Vec<(usize, f32, f32)> = tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.points.is_empty())
        .map(|(idx, t)| {
            let start = t.points.first().unwrap().t_sec;
            let end = t.points.last().unwrap().t_sec + 0.020;
            (idx, start, end)
        })
        .collect();
    track_infos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (track_idx, start_time, end_time) in track_infos {
        let track = &tracks[track_idx];

        // Find a free voice or create new one
        let voice_idx = voice_end_times
            .iter()
            .enumerate()
            .find(|(_, &end)| end + 0.005 <= start_time)
            .map(|(i, _)| i)
            .unwrap_or_else(|| {
                voices.push(Vec::new());
                voice_end_times.push(0.0);
                voices.len() - 1
            });

        // Add silence gap if needed
        let gap = start_time - voice_end_times[voice_idx];
        if gap > 0.0001 {
            let start_freq = track.points.first().map(|p| p.freq_hz).unwrap_or(440.0);
            voices[voice_idx].push(create_silence(gap, start_freq, base_freq));
            voice_end_times[voice_idx] += gap;
        }

        // Add track's PointOps
        let track_ops = create_track_pointops(track, base_freq, fps);
        let track_duration: f32 = track_ops.iter()
            .map(|op| rational_to_f32(op.l))
            .sum();

        for op in track_ops {
            voices[voice_idx].push(op);
        }
        voice_end_times[voice_idx] += track_duration;
    }

    // Calculate length ratio
    let max_length = voices.iter()
        .map(|voice| voice.iter().map(|op| op.l).sum::<Rational64>())
        .max()
        .unwrap_or(Rational64::from_integer(1));

    // Pad all voices to the same length (NormalForm invariant)
    // This prevents audio engine issues when voices end at different times
    for voice in &mut voices {
        let voice_length: Rational64 = voice.iter().map(|op| op.l).sum();
        let padding_needed = max_length - voice_length;
        if padding_needed > Rational64::from_integer(0) {
            // Get last frequency for portamento continuity
            let last_freq = voice.last()
                .map(|op| rational_to_f32(op.fm) * base_freq)
                .unwrap_or(440.0);
            voice.push(create_silence(rational_to_f32(padding_needed), last_freq, base_freq));
        }
    }

    Ok(NormalForm {
        operations: voices,
        length_ratio: max_length,
    })
}

/// Create PointOps for a single track with loudness pre-compensation
fn create_track_pointops(track: &TrackOut, base_freq: f32, fps: usize) -> Vec<PointOp> {
    let mut ops = Vec::new();

    if track.points.len() < 2 {
        return ops;
    }

    let track_end_time = track.points.last().map(|p| p.t_sec).unwrap_or(1.0);

    for i in 0..track.points.len() - 1 {
        let point_a = &track.points[i];
        let point_b = &track.points[i + 1];

        let segment_duration = point_b.t_sec - point_a.t_sec;
        if segment_duration < 0.0001 {
            continue;
        }

        let base_step = 1.0 / fps as f32;
        let num_steps = (segment_duration / base_step).ceil().max(1.0) as usize;
        let actual_step = segment_duration / num_steps as f32;

        for step in 0..num_steps {
            let t = step as f32 / num_steps as f32;
            let current_time = point_a.t_sec + t * segment_duration;

            // Linear interpolation
            let interp_freq = point_a.freq_hz + (point_b.freq_hz - point_a.freq_hz) * t;
            let interp_amp = point_a.amp + (point_b.amp - point_a.amp) * t;

            // Apply fadeout near track end
            let time_to_end = track_end_time - current_time;
            let fade_time = 0.020;
            let fadeout_factor = if time_to_end <= 0.0 {
                0.0
            } else if time_to_end < fade_time {
                time_to_end / fade_time
            } else {
                1.0
            };
            let final_amp = interp_amp * fadeout_factor;

            // Pre-compensate for loudness normalization
            let precomp_amp = final_amp * loudness_precompensation(interp_freq);

            let is_first = i == 0 && step == 0;

            ops.push(PointOp {
                fm: rational_from_f32(interp_freq / base_freq),
                fa: Rational64::from_integer(0),
                g: rational_from_f32(precomp_amp),
                l: rational_from_f32(actual_step),
                pm: Rational64::from_integer(1),
                pa: Rational64::from_integer(0),
                attack: if is_first { rational_from_f32(0.010) } else { Rational64::from_integer(0) },
                decay: Rational64::from_integer(0),
                asr: ASR::Long,
                portamento: rational_from_f32(actual_step),
                osc_type: OscType::Sine { pow: None },
                ..Default::default()
            });
        }
    }

    ops
}

/// Pre-compensation factor for loudness normalization
fn loudness_precompensation(freq_hz: f32) -> f32 {
    if freq_hz < 20.0 {
        return 1.0;
    }
    let exponent = (20.0 * freq_hz.log10() - 40.0) / 10.0;
    2.0_f32.powf(exponent).min(16.0)
}

/// Create a silence PointOp with specified frequency (for portamento continuity)
fn create_silence(duration: f32, freq: f32, base_freq: f32) -> PointOp {
    PointOp {
        fm: rational_from_f32(freq / base_freq),
        fa: Rational64::from_integer(0),
        g: Rational64::from_integer(0),
        l: rational_from_f32(duration),
        pm: Rational64::from_integer(1),
        pa: Rational64::from_integer(0),
        attack: Rational64::from_integer(0),
        decay: Rational64::from_integer(0),
        asr: ASR::Long,
        portamento: Rational64::from_integer(0),
        osc_type: OscType::Sine { pow: None },
        ..Default::default()
    }
}

/// Convert f32 to Rational64
fn rational_from_f32(f: f32) -> Rational64 {
    if !f.is_finite() || f.abs() > 1_000_000.0 {
        return Rational64::from_integer(0);
    }
    let scale = 1_000_000_i64;
    let num = (f * scale as f32) as i64;
    Rational64::new(num, scale)
}

/// Convert Rational64 to f32
fn rational_to_f32(r: Rational64) -> f32 {
    *r.numer() as f32 / *r.denom() as f32
}

/// Get cache file path for given audio file and voice count
fn get_cache_path(audio_path: &str, voices: usize, fps: usize) -> std::path::PathBuf {
    let path = Path::new(audio_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let cache_dir = parent.join(".weresocool_cache");
    cache_dir.join(format!("{}_{}_{}.bin", stem, voices, fps))
}

/// Load NormalForm from cache if valid
fn load_from_cache(audio_path: &str, voices: usize, fps: usize) -> Result<Option<NormalForm>, Error> {
    let cache_path = get_cache_path(audio_path, voices, fps);

    if !cache_path.exists() {
        return Ok(None);
    }

    // Check audio file mtime
    let audio_mtime = fs::metadata(audio_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Read cache (binary)
    let read_start = std::time::Instant::now();
    let cache_data = fs::read(&cache_path)
        .map_err(|e| Error::with_msg(format!("Failed to read cache: {}", e)))?;
    eprintln!("[FromSound] Cache file read: {:?} ({} bytes)", read_start.elapsed(), cache_data.len());

    let parse_start = std::time::Instant::now();
    let cache: FromSoundCache = bincode::deserialize(&cache_data)
        .map_err(|e| Error::with_msg(format!("Failed to parse cache: {}", e)))?;
    eprintln!("[FromSound] Bincode deserialize: {:?}", parse_start.elapsed());

    // Validate cache
    if cache.audio_path != audio_path || cache.audio_mtime != audio_mtime || cache.voices != voices {
        return Ok(None);
    }

    // Reconstruct NormalForm
    let reconstruct_start = std::time::Instant::now();
    let operations: Vec<Vec<PointOp>> = cache.operations
        .iter()
        .map(|voice| voice.iter().map(|op| op.to_point_op()).collect())
        .collect();
    eprintln!("[FromSound] Reconstruct NormalForm: {:?} ({} voices, {} total ops)",
        reconstruct_start.elapsed(),
        operations.len(),
        operations.iter().map(|v| v.len()).sum::<usize>());

    let length_ratio = Rational64::new(cache.length_ratio_num, cache.length_ratio_denom);

    Ok(Some(NormalForm {
        operations,
        length_ratio,
    }))
}

/// Save NormalForm to cache
fn save_to_cache(audio_path: &str, voices: usize, fps: usize, nf: &NormalForm) -> Result<(), Error> {
    let cache_path = get_cache_path(audio_path, voices, fps);

    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::with_msg(format!("Failed to create cache directory: {}", e)))?;
    }

    // Get audio mtime
    let audio_mtime = fs::metadata(audio_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Serialize operations
    let operations: Vec<Vec<SerializedPointOp>> = nf.operations
        .iter()
        .map(|voice| voice.iter().map(|op| op.into()).collect())
        .collect();

    let cache = FromSoundCache {
        audio_path: audio_path.to_string(),
        audio_mtime,
        voices,
        operations,
        length_ratio_num: *nf.length_ratio.numer(),
        length_ratio_denom: *nf.length_ratio.denom(),
    };

    let cache_data = bincode::serialize(&cache)
        .map_err(|e| Error::with_msg(format!("Failed to serialize cache: {}", e)))?;

    fs::write(&cache_path, cache_data)
        .map_err(|e| Error::with_msg(format!("Failed to write cache: {}", e)))?;

    Ok(())
}

// ============================================================================
// FromSoundYin: YIN-based monophonic pitch detection
// ============================================================================

use weresocool_analyze::{Analyze, DetectionResult};

/// Cache entry for FromSoundYin analysis results
#[derive(serde::Serialize, serde::Deserialize)]
struct FromSoundYinCache {
    audio_path: String,
    audio_mtime: u64,
    fps: usize,
    operations: Vec<SerializedPointOp>,
    length_ratio_num: i64,
    length_ratio_denom: i64,
}

/// Main entry point: analyze audio file using YIN and return NormalForm
pub fn from_sound_yin_to_normalform(path: &str, fps: usize) -> Result<NormalForm, Error> {
    // Validate file exists
    if !Path::new(path).exists() {
        return Err(Error::with_msg(format!(
            "FromSoundYin: audio file not found: '{}'\nMake sure the path is relative to the .socool file location.",
            path
        )));
    }

    // Try to load from cache first
    if let Some(nf) = load_yin_from_cache(path, fps)? {
        return Ok(nf);
    }

    // Read the audio file
    let (sample_rate, samples) = read_wav_mono(path)
        .map_err(|e| Error::with_msg(format!("Failed to read WAV file: {}", e)))?;

    // Convert to NormalForm using YIN
    let nf = yin_to_normalform(&samples, sample_rate, fps)?;

    // Save to cache
    save_yin_to_cache(path, fps, &nf)?;

    Ok(nf)
}

/// Convert audio samples to NormalForm using YIN pitch detection
fn yin_to_normalform(samples: &[f32], sample_rate: u32, fps: usize) -> Result<NormalForm, Error> {
    let base_freq = 440.0_f32;
    let sr = sample_rate as f32;

    // Frame parameters
    let frame_size = (sr / fps as f32) as usize;
    let hop_size = frame_size; // Non-overlapping
    let threshold = 0.2; // YIN threshold

    // Minimum buffer size for YIN (needs enough samples for low frequencies)
    let min_buffer_size = (sr / 60.0) as usize * 2; // Support down to 60Hz
    let buffer_size = frame_size.max(min_buffer_size);

    let mut ops: Vec<PointOp> = Vec::new();
    let frame_duration = 1.0 / fps as f32;

    let total_frames = samples.len() / hop_size;

    for frame_idx in 0..total_frames {
        let start = frame_idx * hop_size;
        let end = (start + buffer_size).min(samples.len());

        if end - start < buffer_size / 2 {
            break; // Not enough samples
        }

        // Extract frame and run YIN
        let mut frame: Vec<f32> = samples[start..end].to_vec();
        let result: DetectionResult = frame.analyze(sr, threshold);

        // Skip if no valid pitch detected
        let freq = if result.frequency > 60.0 && result.frequency < 2000.0 && result.probability > 0.0 {
            result.frequency
        } else {
            0.0 // Silence
        };

        let gain = if freq > 0.0 {
            result.gain * loudness_precompensation(freq)
        } else {
            0.0
        };

        let is_first = frame_idx == 0;

        ops.push(PointOp {
            fm: if freq > 0.0 { rational_from_f32(freq / base_freq) } else { Rational64::from_integer(1) },
            fa: Rational64::from_integer(0),
            g: rational_from_f32(gain),
            l: rational_from_f32(frame_duration),
            pm: Rational64::from_integer(1),
            pa: Rational64::from_integer(0),
            attack: if is_first { rational_from_f32(0.010) } else { Rational64::from_integer(0) },
            decay: Rational64::from_integer(0),
            asr: ASR::Long,
            portamento: rational_from_f32(frame_duration),
            osc_type: OscType::Sine { pow: None },
            ..Default::default()
        });
    }

    // Calculate length ratio
    let total_length: Rational64 = ops.iter().map(|op| op.l).sum();

    Ok(NormalForm {
        operations: vec![ops], // Single voice
        length_ratio: total_length,
    })
}

/// Get cache file path for YIN analysis
fn get_yin_cache_path(audio_path: &str, fps: usize) -> std::path::PathBuf {
    let path = Path::new(audio_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let cache_dir = parent.join(".weresocool_cache");
    cache_dir.join(format!("{}_yin_{}.json", stem, fps))
}

/// Load NormalForm from YIN cache if valid
fn load_yin_from_cache(audio_path: &str, fps: usize) -> Result<Option<NormalForm>, Error> {
    let cache_path = get_yin_cache_path(audio_path, fps);

    if !cache_path.exists() {
        return Ok(None);
    }

    // Check audio file mtime
    let audio_mtime = fs::metadata(audio_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Read cache
    let cache_data = fs::read_to_string(&cache_path)
        .map_err(|e| Error::with_msg(format!("Failed to read cache: {}", e)))?;

    let cache: FromSoundYinCache = serde_json::from_str(&cache_data)
        .map_err(|e| Error::with_msg(format!("Failed to parse cache: {}", e)))?;

    // Validate cache
    if cache.audio_path != audio_path || cache.audio_mtime != audio_mtime || cache.fps != fps {
        return Ok(None);
    }

    // Reconstruct NormalForm
    let operations: Vec<PointOp> = cache.operations
        .iter()
        .map(|op| op.to_point_op())
        .collect();

    let length_ratio = Rational64::new(cache.length_ratio_num, cache.length_ratio_denom);

    Ok(Some(NormalForm {
        operations: vec![operations],
        length_ratio,
    }))
}

/// Save NormalForm to YIN cache
fn save_yin_to_cache(audio_path: &str, fps: usize, nf: &NormalForm) -> Result<(), Error> {
    let cache_path = get_yin_cache_path(audio_path, fps);

    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::with_msg(format!("Failed to create cache directory: {}", e)))?;
    }

    // Get audio mtime
    let audio_mtime = fs::metadata(audio_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Serialize operations (single voice)
    let operations: Vec<SerializedPointOp> = nf.operations
        .first()
        .map(|voice| voice.iter().map(|op| op.into()).collect())
        .unwrap_or_default();

    let cache = FromSoundYinCache {
        audio_path: audio_path.to_string(),
        audio_mtime,
        fps,
        operations,
        length_ratio_num: *nf.length_ratio.numer(),
        length_ratio_denom: *nf.length_ratio.denom(),
    };

    let cache_data = serde_json::to_string(&cache)
        .map_err(|e| Error::with_msg(format!("Failed to serialize cache: {}", e)))?;

    fs::write(&cache_path, cache_data)
        .map_err(|e| Error::with_msg(format!("Failed to write cache: {}", e)))?;

    Ok(())
}
