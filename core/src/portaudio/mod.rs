// Audio backend architecture
pub mod backend;
pub mod portaudio_backend;
pub mod cpal_backend;

// Public API
pub use self::backend::{AudioBackend, BackendConfig};
pub use self::portaudio_backend::{create_portaudio_stream, create_portaudio_duplex_stream, create_portaudio_duplex_stream_with_capture, create_portaudio_duplex_mix_stream, create_portaudio_input_capture_stream, play_samples_blocking, CapturedBlock, PortAudioBackend};
pub use self::cpal_backend::{create_cpal_stream, CpalBackend};
