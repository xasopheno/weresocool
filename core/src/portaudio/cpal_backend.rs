/// CPAL audio backend
///
/// Modern cross-platform audio library backend.
/// Pure Rust implementation, better maintained than PortAudio.

use super::backend::{AudioBackend, BackendConfig};
use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_error::Error;
use weresocool_instrument::{Offset, StereoWaveform};
use weresocool_shared::Settings;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    StreamConfig,
};
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

pub struct CpalBackend {
    host: cpal::Host,
}

impl AudioBackend for CpalBackend {
    type Stream = cpal::Stream;

    fn new() -> Result<Self, Error> {
        let host = cpal::default_host();
        Ok(Self { host })
    }

    fn create_stream(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        config: BackendConfig,
    ) -> Result<Self::Stream, Error> {
        if config.mic_input {
            // TODO: Implement duplex mode for CPAL
            // For now, fall back to output-only
            self.create_output_stream(render_manager, config.use_lookahead)
        } else {
            self.create_output_stream(render_manager, config.use_lookahead)
        }
    }
}

impl CpalBackend {
    fn create_output_stream(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        use_lookahead: bool,
    ) -> Result<cpal::Stream, Error> {
        let settings = Settings::global();
        let device = self
            .host
            .default_output_device()
            .ok_or_else(|| Error::with_msg("Failed to get default output device"))?;

        let config = StreamConfig {
            channels: settings.channels as u16,
            sample_rate: cpal::SampleRate(settings.sample_rate as u32),
            buffer_size: cpal::BufferSize::Fixed(settings.buffer_size as u32),
        };

        if use_lookahead {
            // Get stream_active flag for fast-path check in callback
            let stream_active = render_manager.lock().unwrap().stream_active();

            // Use background rendering with lookahead buffers
            let _render_thread =
                RenderManager::start_background_rendering(Arc::clone(&render_manager), Arc::clone(&stream_active));

            let stream = device
                .build_output_stream(
                    &config,
                    move |output: &mut [f32], _| {
                        // Fast path: if stream inactive, just zero buffer - NO locks, NO processing
                        if !stream_active.load(Ordering::Relaxed) {
                            for sample in output.iter_mut() {
                                *sample = 0.0;
                            }
                            return;
                        }

                        // Check for VisReady or timeout before reading buffer
                        if let Ok(mut rm) = render_manager.lock() {
                            rm.check_vis_ready();
                        }

                        // Try to pop a pre-rendered buffer from the queue
                        // Handle lock failure gracefully - output silence instead of panicking
                        let buffer = match render_manager.lock() {
                            Ok(rm) => rm.pop_buffer(),
                            Err(e) => {
                                eprintln!("ERROR: RenderManager lock poisoned in CPAL audio callback: {}", e);
                                None
                            }
                        };

                        if let Some(prerendered) = buffer {
                            new_write_output_buffer(output, prerendered.waveform, prerendered.ramp);
                        } else {
                            // No buffer available (underrun or lock failure) - output silence
                            write_output_buffer(output, StereoWaveform::new(settings.buffer_size));
                        }
                    },
                    |err| eprintln!("CPAL output stream error: {}", err),
                    None,
                )
                .map_err(|e| Error::with_msg(format!("Failed to build output stream: {}", e)))?;

            Ok(stream)
        } else {
            // Get stream_active flag for fast-path check in callback
            let stream_active = render_manager.lock().unwrap().stream_active();

            // Render directly on audio thread (no lookahead)
            let stream = device
                .build_output_stream(
                    &config,
                    move |output: &mut [f32], _| {
                        // Fast path: if stream inactive, just zero buffer - NO locks, NO processing
                        if !stream_active.load(Ordering::Relaxed) {
                            for sample in output.iter_mut() {
                                *sample = 0.0;
                            }
                            return;
                        }

                        // Handle lock failure gracefully - output silence instead of panicking
                        let batch = match render_manager.lock() {
                            Ok(mut rm) => {
                                // Check for VisReady or timeout before rendering
                                rm.check_vis_ready();

                                rm.read(
                                    settings.buffer_size,
                                    Offset {
                                        freq: 1.0,
                                        gain: 1.0,
                                    },
                                )
                            },
                            Err(e) => {
                                eprintln!("ERROR: RenderManager lock poisoned in CPAL audio callback: {}", e);
                                None
                            }
                        };

                        if let Some((b, ramp, _ops)) = batch {
                            new_write_output_buffer(output, b, ramp);
                        } else {
                            write_output_buffer(output, StereoWaveform::new(settings.buffer_size));
                        }
                    },
                    |err| eprintln!("CPAL output stream error: {}", err),
                    None,
                )
                .map_err(|e| Error::with_msg(format!("Failed to build output stream: {}", e)))?;

            Ok(stream)
        }
    }
}

/// Convenience function to create a CPAL stream
pub fn create_cpal_stream(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<cpal::Stream, Error> {
    let backend = CpalBackend::new()?;
    let config = BackendConfig {
        mic_input: false,
        use_lookahead: Settings::global().lookahead_buffers > 0,
    };
    backend.create_stream(render_manager, config)
}
