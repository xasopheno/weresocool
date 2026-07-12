// Native only: the sidecar store is filesystem + WAV I/O, and
// transcription rides on from_sound (configured out on wasm).
#[cfg(not(target_arch = "wasm32"))]
pub mod daw;
pub mod events;
pub mod generation;
pub mod interpretable;
pub mod manager;
#[cfg(feature = "app")]
pub mod portaudio;
pub mod renderable;
pub mod ui;
pub mod write;
