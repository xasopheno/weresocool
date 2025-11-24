/// Generic audio backend trait
///
/// This trait defines the interface for audio playback backends.
/// Implementations can use PortAudio, CPAL, or any other audio library.

use crate::manager::RenderManager;
use std::sync::{Arc, Mutex};
use weresocool_error::Error;

/// Configuration for audio backends
#[derive(Clone, Debug)]
pub struct BackendConfig {
    /// Whether to enable microphone input (duplex mode)
    pub mic_input: bool,

    /// Whether to use lookahead buffering (background rendering)
    pub use_lookahead: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            mic_input: false,
            use_lookahead: true,
        }
    }
}

/// Audio backend trait
///
/// Implementors must define their stream type and how to create it
pub trait AudioBackend {
    /// The type of audio stream this backend produces
    type Stream;

    /// Create a new instance of this backend
    fn new() -> Result<Self, Error>
    where
        Self: Sized;

    /// Create an audio output stream
    fn create_stream(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        config: BackendConfig,
    ) -> Result<Self::Stream, Error>;
}
