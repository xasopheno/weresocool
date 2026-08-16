//! Tiny-DAW data layer (bevy-free): the sidecar store, layer/monitor source
//! injection, take promotion, and WAV → NormalForm transcription.
//!
//! The interactive half (capture loop, reducer, egui panel) lives in the
//! kintaro crate; everything here is pure data + source transforms, shared by
//! native watch AND audio-only hosts like the Logic AU plugin — so recorded
//! layers play wherever the composition loads.

pub mod inject;
pub mod push;
pub mod store;
// Transcription rides on from_sound, which only the desktop feature sets
// carry (weresocool_ast/app) — the wasm/mobile set builds this module
// without it, and legacy WAV-only recordings simply stay pending there.
#[cfg(any(feature = "app", feature = "windows"))]
pub mod transcribe;

pub use inject::{apply_injection, LayerInjection};
pub use store::{DawStore, Layer, Manifest, Recording, Take};

use std::path::Path;
use weresocool_ast::RecordingRegistry;

/// Build the full DAW injection for a composition: the source transform that
/// overlays WIP layers + the registry that resolves their `Perform` events (and
/// any named recordings).
pub fn injection_for(socool_path: &Path) -> LayerInjection {
    let store = DawStore::for_composition(socool_path);
    // The slot name = the palette's name (`palette perf { … }`), or `daw` if the
    // palette is unnamed. Read from the raw source (apply_injection runs before
    // the palette strip, so the name is still present in `main` too).
    let slot = std::fs::read_to_string(socool_path)
        .ok()
        .and_then(|s| weresocool_parser::palette::extract_palette(&s).name)
        .unwrap_or_else(|| "daw".to_string());
    let mut injection = inject::build_layer_injection(&store, &slot);
    // Named recordings (authored `Perform("name")` / pre-arm path) also resolve.
    for (name, nf) in build_recordings_registry(&store) {
        injection.registry.insert(name, nf);
    }
    injection
}

/// Transcribe every named recording in the store into a `RecordingRegistry`
/// ready to seed `Perform("name")` resolution. Transcription failures are
/// logged and skipped (that name then renders silent / stays pending).
pub fn build_recordings_registry(store: &DawStore) -> RecordingRegistry {
    let mut registry = RecordingRegistry::new();
    for rec in store.recordings() {
        // Prefer the frozen events (faithful); fall back to re-transcribing the
        // WAV for legacy recordings without events.
        if !rec.events.is_empty() {
            registry.insert(rec.name.clone(), inject::events_to_normalform(&rec.events));
            continue;
        }
        #[cfg(any(feature = "app", feature = "windows"))]
        {
            let Some(path) = store.recording_wav(&rec.name) else {
                continue;
            };
            match transcribe::transcribe_wav(&path, rec.fps) {
                Ok(nf) => {
                    registry.insert(rec.name.clone(), nf);
                }
                Err(e) => eprintln!("[daw] transcribe '{}' failed: {}", rec.name, e),
            }
        }
        #[cfg(not(any(feature = "app", feature = "windows")))]
        eprintln!(
            "[daw] '{}' has no frozen events and this build cannot transcribe",
            rec.name
        );
    }
    registry
}
