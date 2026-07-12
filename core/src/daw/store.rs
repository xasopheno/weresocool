//! On-disk store for the tiny DAW.
//!
//! Everything for one composition lives in a sidecar directory named after the
//! full filename so siblings never collide: `song.socool` → `song.socool.daw/`.
//!
//! ```text
//! song.socool.daw/
//!   manifest.json          tracks, takes, recordings, active selection
//!   takes/<track>/N.wav    raw loop-record takes (choose / delete)
//!   recordings/<name>.wav  takes promoted via "push" — referenced by Perform("name")
//! ```
//!
//! Takes are the raw audio captured one-loop-at-a-time. A take becomes a named
//! `Recording` when the user pushes it into the piece. We keep the audio so the
//! transcription (YIN now, pluggable later) can be re-derived without
//! re-recording.

use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// One raw loop-record take, captured against an armed voice (`track`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Take {
    pub id: u64,
    /// The armed voice this take was recorded against.
    pub track: String,
    /// Path relative to the store root, e.g. `takes/thing/3.wav`.
    pub wav: String,
    pub sample_rate: u32,
    pub frames: usize,
}

/// A committed recording, referenced from source as `Perform("name")`. Carries
/// the frozen events (preferred) and keeps the raw audio for re-analysis.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Recording {
    pub name: String,
    /// Path relative to the store root, e.g. `recordings/saxophone.wav`.
    pub wav: String,
    pub sample_rate: u32,
    pub frames: usize,
    /// Frames-per-second used when transcribing this recording's audio.
    pub fps: usize,
    /// Frozen events (faithful). If present, used directly; else the WAV is
    /// re-transcribed.
    #[serde(default)]
    pub events: Vec<Event>,
}

/// One frozen note of a layer's performance — the faithful mic contour. Stored
/// as plain numbers; the `NormalForm` is rebuilt at injection time (needs the
/// composition's `f_basis`, and `NormalForm` isn't `Serialize`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    /// Frequency multiplier relative to the composition's fundamental
    /// (`detected_hz / f_basis`), computed at capture. 0 gain marks silence.
    pub fm: f64,
    /// Detected gain (0..1).
    pub gain: f64,
    /// Note length in seconds.
    pub secs: f64,
}

/// A recorded layer: a frozen performance played through a palette sound.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Layer {
    pub id: u64,
    /// Display name (also the `Perform` base name on commit).
    pub name: String,
    /// Which palette instrument this layer plays through.
    pub palette_sound: String,
    pub sample_rate: u32,
    pub frames: usize,
    /// Raw audio path relative to the store root (`layers/<id>.wav`), kept for
    /// later re-analysis.
    pub wav: String,
    /// The frozen contour (faithful mic events).
    pub events: Vec<Event>,
    pub muted: bool,
    /// Output gain multiplier (injected as `| Gm <volume>`).
    #[serde(default = "default_volume")]
    pub volume: f64,
    #[serde(default)]
    pub soloed: bool,
    /// Click-to-place origin (world `[x, y, z]`) captured when this take was
    /// recorded. `Some` → the layer's marks draw at this fixed point (the audio
    /// is unaffected). `None` → marks follow the note's pan/pitch. See
    /// `AppState::brush_origin`.
    #[serde(default)]
    pub origin: Option<[f32; 3]>,
}

fn default_volume() -> f64 {
    1.0
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub takes: Vec<Take>,
    pub recordings: Vec<Recording>,
    #[serde(default)]
    pub layers: Vec<Layer>,
    /// Palette sound currently armed for live monitoring (a `Follow`-wrapped
    /// voice of this sound is overlaid so you hear/see your mic through it
    /// before recording). `None` = no monitor.
    #[serde(default)]
    pub armed_sound: Option<String>,
    /// Mic-sensitivity slider position — remembered across sessions/reloads.
    #[serde(default = "default_mic_gain")]
    pub mic_gain: f32,
    /// Round-trip record latency compensation, in milliseconds. Recorded layers
    /// are anchored this much EARLIER in the loop so what you sang lands where
    /// you heard it (output + input + analysis latency). `None` = derive an
    /// automatic estimate from the audio params; `Some(ms)` = manual override.
    #[serde(default)]
    pub record_latency_ms: Option<f32>,
}

fn default_mic_gain() -> f32 {
    4.0
}

impl Default for Manifest {
    fn default() -> Self {
        Manifest {
            takes: Vec::new(),
            recordings: Vec::new(),
            layers: Vec::new(),
            armed_sound: None,
            mic_gain: default_mic_gain(),
            record_latency_ms: None,
        }
    }
}

/// Handle to a composition's DAW sidecar directory plus its loaded manifest.
#[derive(Clone, Debug)]
pub struct DawStore {
    root: PathBuf,
    pub manifest: Manifest,
}

impl DawStore {
    /// Derive the sidecar root for a `.socool` path and load its manifest
    /// (empty if the directory doesn't exist yet). Never creates anything on
    /// disk — that happens lazily on the first write.
    pub fn for_composition(socool_path: &Path) -> Self {
        let root = sidecar_root(socool_path);
        Self::load(root)
    }

    pub fn load(root: PathBuf) -> Self {
        let manifest = std::fs::read_to_string(root.join("manifest.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        DawStore { root, manifest }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn save(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let json = serde_json::to_string_pretty(&self.manifest)?;
        std::fs::write(self.root.join("manifest.json"), json)
    }

    fn next_take_id(&self) -> u64 {
        self.manifest.takes.iter().map(|t| t.id).max().map_or(1, |m| m + 1)
    }

    /// Write a captured loop buffer as a new take for `track` and persist it.
    pub fn add_take(
        &mut self,
        track: &str,
        samples: &[f32],
        sample_rate: u32,
    ) -> io::Result<Take> {
        let id = self.next_take_id();
        let rel = format!("takes/{}/{}.wav", track, id);
        let abs = self.root.join(&rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_mono_wav(&abs, samples, sample_rate)?;

        let take = Take {
            id,
            track: track.to_string(),
            wav: rel,
            sample_rate,
            frames: samples.len(),
        };
        self.manifest.takes.push(take.clone());
        self.save()?;
        Ok(take)
    }

    pub fn takes_for(&self, track: &str) -> Vec<&Take> {
        self.manifest.takes.iter().filter(|t| t.track == track).collect()
    }

    pub fn take(&self, id: u64) -> Option<&Take> {
        self.manifest.takes.iter().find(|t| t.id == id)
    }

    /// Remove a take (file + manifest entry) and persist.
    pub fn delete_take(&mut self, id: u64) -> io::Result<()> {
        if let Some(pos) = self.manifest.takes.iter().position(|t| t.id == id) {
            let abs = self.root.join(&self.manifest.takes[pos].wav);
            let _ = std::fs::remove_file(abs); // best-effort
            self.manifest.takes.remove(pos);
            self.save()?;
        }
        Ok(())
    }

    /// Promote a take to a named recording: copy its audio into
    /// `recordings/<name>.wav` and record it in the manifest. Replaces any
    /// existing recording with the same name. Returns the new `Recording`.
    pub fn promote(&mut self, take_id: u64, name: &str, fps: usize) -> io::Result<Recording> {
        let take = self
            .take(take_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "take not found"))?
            .clone();

        let rel = format!("recordings/{}.wav", name);
        let abs = self.root.join(&rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(self.root.join(&take.wav), &abs)?;

        let recording = Recording {
            name: name.to_string(),
            wav: rel,
            sample_rate: take.sample_rate,
            frames: take.frames,
            fps,
            events: Vec::new(),
        };
        self.manifest.recordings.retain(|r| r.name != name);
        self.manifest.recordings.push(recording.clone());
        self.save()?;
        Ok(recording)
    }

    pub fn recordings(&self) -> &[Recording] {
        &self.manifest.recordings
    }

    pub fn recording(&self, name: &str) -> Option<&Recording> {
        self.manifest.recordings.iter().find(|r| r.name == name)
    }

    /// Absolute path to a recording's WAV, if it exists in the manifest.
    pub fn recording_wav(&self, name: &str) -> Option<PathBuf> {
        self.recording(name).map(|r| self.root.join(&r.wav))
    }

    /// Absolute path to a take's WAV.
    pub fn take_wav(&self, id: u64) -> Option<PathBuf> {
        self.take(id).map(|t| self.root.join(&t.wav))
    }

    // ── Layers (the looper) ────────────────────────────────────────────────

    fn next_layer_id(&self) -> u64 {
        self.manifest.layers.iter().map(|l| l.id).max().map_or(1, |m| m + 1)
    }

    /// Add a recorded layer: persist its raw audio and frozen events.
    pub fn add_layer(
        &mut self,
        name: &str,
        palette_sound: &str,
        events: Vec<Event>,
        samples: &[f32],
        sample_rate: u32,
        origin: Option<[f32; 3]>,
    ) -> io::Result<Layer> {
        let id = self.next_layer_id();
        let rel = format!("layers/{}.wav", id);
        let abs = self.root.join(&rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_mono_wav(&abs, samples, sample_rate)?;

        let layer = Layer {
            id,
            name: name.to_string(),
            palette_sound: palette_sound.to_string(),
            sample_rate,
            frames: samples.len(),
            wav: rel,
            events,
            muted: false,
            volume: 1.0,
            soloed: false,
            origin,
        };
        self.manifest.layers.push(layer.clone());
        self.save()?;
        Ok(layer)
    }

    pub fn layers(&self) -> &[Layer] {
        &self.manifest.layers
    }

    pub fn delete_layer(&mut self, id: u64) -> io::Result<()> {
        if let Some(pos) = self.manifest.layers.iter().position(|l| l.id == id) {
            let abs = self.root.join(&self.manifest.layers[pos].wav);
            let _ = std::fs::remove_file(abs);
            self.manifest.layers.remove(pos);
            self.save()?;
        }
        Ok(())
    }

    pub fn set_layer_muted(&mut self, id: u64, muted: bool) -> io::Result<()> {
        if let Some(l) = self.manifest.layers.iter_mut().find(|l| l.id == id) {
            l.muted = muted;
            self.save()?;
        }
        Ok(())
    }

    /// Re-brush a layer: perform its frozen gesture through a DIFFERENT palette
    /// sound. The recorded events are untouched — only the decorator changes.
    pub fn set_layer_sound(&mut self, id: u64, sound: &str) -> io::Result<()> {
        if let Some(l) = self.manifest.layers.iter_mut().find(|l| l.id == id) {
            l.palette_sound = sound.to_string();
            self.save()?;
        }
        Ok(())
    }

    pub fn set_layer_volume(&mut self, id: u64, volume: f64) -> io::Result<()> {
        if let Some(l) = self.manifest.layers.iter_mut().find(|l| l.id == id) {
            l.volume = volume;
            self.save()?;
        }
        Ok(())
    }

    /// Set a layer's volume in memory only — NO disk write. Pair with `save()`
    /// on fader drag-release so a continuous drag (a `changed()` per frame)
    /// doesn't rewrite the manifest 60×/second. The live audio gain reads the
    /// in-memory value, so the fader stays responsive without the I/O.
    pub fn set_layer_volume_mem(&mut self, id: u64, volume: f64) {
        if let Some(l) = self.manifest.layers.iter_mut().find(|l| l.id == id) {
            l.volume = volume;
        }
    }

    pub fn set_layer_soloed(&mut self, id: u64, soloed: bool) -> io::Result<()> {
        if let Some(l) = self.manifest.layers.iter_mut().find(|l| l.id == id) {
            l.soloed = soloed;
            self.save()?;
        }
        Ok(())
    }

    pub fn rename_layer(&mut self, id: u64, name: &str) -> io::Result<()> {
        if let Some(l) = self.manifest.layers.iter_mut().find(|l| l.id == id) {
            l.name = name.to_string();
            self.save()?;
        }
        Ok(())
    }

    /// Delete the most recently added layer (undo). Returns its id if any.
    pub fn undo_last_layer(&mut self) -> io::Result<Option<u64>> {
        let Some(id) = self.manifest.layers.iter().map(|l| l.id).max() else {
            return Ok(None);
        };
        self.delete_layer(id)?;
        Ok(Some(id))
    }

    /// Whether any layer is soloed (then only soloed layers sound).
    pub fn any_soloed(&self) -> bool {
        self.manifest.layers.iter().any(|l| l.soloed)
    }

    /// Commit: promote a layer to a named recording (copy its audio, keep its
    /// events) so `Perform("name")` resolves it after the layer is removed.
    pub fn add_recording_from_layer(&mut self, layer_id: u64, name: &str) -> io::Result<()> {
        let layer = self
            .manifest
            .layers
            .iter()
            .find(|l| l.id == layer_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "layer not found"))?
            .clone();
        let rel = format!("recordings/{}.wav", name);
        let abs = self.root.join(&rel);
        if let Some(p) = abs.parent() {
            std::fs::create_dir_all(p)?;
        }
        let _ = std::fs::copy(self.root.join(&layer.wav), &abs);
        self.manifest.recordings.retain(|r| r.name != name);
        self.manifest.recordings.push(Recording {
            name: name.to_string(),
            wav: rel,
            sample_rate: layer.sample_rate,
            frames: layer.frames,
            fps: 30,
            events: layer.events,
        });
        self.save()
    }

    /// Remove all WIP layers (after commit).
    pub fn clear_layers(&mut self) -> io::Result<()> {
        for l in &self.manifest.layers {
            let _ = std::fs::remove_file(self.root.join(&l.wav));
        }
        self.manifest.layers.clear();
        self.save()
    }

    pub fn armed_sound(&self) -> Option<&str> {
        self.manifest.armed_sound.as_deref()
    }

    pub fn mic_gain(&self) -> f32 {
        self.manifest.mic_gain
    }

    pub fn set_mic_gain(&mut self, gain: f32) -> io::Result<()> {
        self.manifest.mic_gain = gain;
        self.save()
    }

    pub fn set_armed_sound(&mut self, sound: Option<String>) -> io::Result<()> {
        self.manifest.armed_sound = sound;
        self.save()
    }

    /// Manual record-latency override in ms (`None` = auto). Persisted.
    pub fn record_latency_ms(&self) -> Option<f32> {
        self.manifest.record_latency_ms
    }

    pub fn set_record_latency_ms(&mut self, ms: Option<f32>) -> io::Result<()> {
        self.manifest.record_latency_ms = ms;
        self.save()
    }
}

/// `song.socool` → `song.socool.daw/` (sibling of the composition file).
pub fn sidecar_root(socool_path: &Path) -> PathBuf {
    let file = socool_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "composition.socool".to_string());
    let dir = format!("{}.daw", file);
    match socool_path.parent() {
        Some(parent) => parent.join(dir),
        None => PathBuf::from(dir),
    }
}

/// Write mono f32 samples as a 32-bit float WAV.
pub fn write_mono_wav(path: &Path, samples: &[f32], sample_rate: u32) -> io::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    for &s in samples {
        writer
            .write_sample(s)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    }
    writer
        .finalize()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// Read a WAV as mono f32 (averages channels if stereo). Returns
/// `(samples, sample_rate)`.
pub fn read_mono_wav(path: &Path) -> io::Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let spec = reader.spec();
    let channels = spec.channels.max(1) as usize;

    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.unwrap_or(0.0))
            .collect(),
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap_or(0) as f32 / max)
                .collect()
        }
    };

    let mono = if channels <= 1 {
        interleaved
    } else {
        interleaved
            .chunks(channels)
            .map(|c| c.iter().sum::<f32>() / channels as f32)
            .collect()
    };
    Ok((mono, spec.sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_comp(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join("kintaro_daw_tests").join(name);
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base.join("song.socool")
    }

    #[test]
    fn sidecar_uses_full_filename() {
        let root = sidecar_root(Path::new("/a/b/song.socool"));
        assert_eq!(root, PathBuf::from("/a/b/song.socool.daw"));
    }

    #[test]
    fn take_roundtrip_and_promote() {
        let comp = tmp_comp("daw_store_test");
        let mut store = DawStore::for_composition(&comp);

        let samples: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) - 0.5).collect();
        let take = store.add_take("thing", &samples, 44_100).unwrap();
        assert_eq!(store.takes_for("thing").len(), 1);

        // WAV round-trips.
        let (back, sr) = read_mono_wav(&store.take_wav(take.id).unwrap()).unwrap();
        assert_eq!(sr, 44_100);
        assert_eq!(back.len(), samples.len());
        assert!((back[10] - samples[10]).abs() < 1e-6);

        // Reload from disk: manifest persisted.
        let reloaded = DawStore::for_composition(&comp);
        assert_eq!(reloaded.takes_for("thing").len(), 1);

        // Promote → named recording exists and points at a real WAV.
        let mut store = reloaded;
        let rec = store.promote(take.id, "saxophone", 30).unwrap();
        assert_eq!(rec.name, "saxophone");
        assert!(store.recording_wav("saxophone").unwrap().exists());

        // Delete the take; recording survives.
        store.delete_take(take.id).unwrap();
        assert_eq!(store.takes_for("thing").len(), 0);
        assert_eq!(store.recordings().len(), 1);
    }
}
