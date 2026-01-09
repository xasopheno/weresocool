/// PortAudio audio backend
///
/// Consolidates the functionality of legacy backend files.
/// Supports both lookahead buffering and direct rendering modes.

use super::backend::{AudioBackend, BackendConfig};
use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_analyze::{Analyze, DetectionResult};
use weresocool_error::Error;
use weresocool_instrument::{Offset, StereoWaveform};
use weresocool_portaudio as pa;
use weresocool_ring_buffer::RingBuffer;
use weresocool_shared::Settings;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

pub struct PortAudioBackend {
    pa: pa::PortAudio,
}

impl AudioBackend for PortAudioBackend {
    type Stream = pa::Stream<pa::NonBlocking, pa::Output<f32>>;

    fn new() -> Result<Self, Error> {
        let pa = pa::PortAudio::new()?;
        Ok(Self { pa })
    }

    fn create_stream(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        config: BackendConfig,
    ) -> Result<Self::Stream, Error> {
        if config.mic_input {
            // NOTE: Duplex mode uses a different stream type, so it's handled by create_portaudio_duplex_stream()
            // For the trait implementation, we fall back to output-only
            self.create_output_stream(render_manager, config.use_lookahead)
        } else {
            self.create_output_stream(render_manager, config.use_lookahead)
        }
    }
}

impl PortAudioBackend {
    fn create_output_stream(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        use_lookahead: bool,
    ) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
        let output_settings = self.get_output_settings()?;

        if use_lookahead {
            // Get stream_active flag for fast-path check in callback
            let stream_active = render_manager.lock().unwrap().stream_active();

            // Use background rendering with lookahead buffers
            let _render_thread =
                RenderManager::start_background_rendering(Arc::clone(&render_manager), Arc::clone(&stream_active));

            let stream = self.pa.open_non_blocking_stream(output_settings, move |args| {
                // Fast path: if stream inactive, just zero buffer - NO locks, NO processing
                if !stream_active.load(Ordering::Relaxed) {
                    for sample in args.buffer.iter_mut() {
                        *sample = 0.0;
                    }
                    return pa::Continue;
                }

                // Check for VisReady or timeout before reading buffer
                if let Ok(mut rm) = render_manager.lock() {
                    rm.check_vis_ready();
                }

                // Handle lock failure gracefully - output silence instead of panicking
                let buffer = match render_manager.lock() {
                    Ok(rm) => rm.pop_buffer(),
                    Err(e) => {
                        eprintln!("ERROR: RenderManager lock poisoned in audio callback: {}", e);
                        None
                    }
                };

                if let Some(prerendered) = buffer {
                    new_write_output_buffer(args.buffer, prerendered.waveform, prerendered.ramp);
                    pa::Continue
                } else {
                    write_output_buffer(
                        args.buffer,
                        StereoWaveform::new(Settings::global().buffer_size),
                    );
                    pa::Continue
                }
            })?;

            Ok(stream)
        } else {
            // Get stream_active flag for fast-path check in callback
            let stream_active = render_manager.lock().unwrap().stream_active();

            // Render directly on audio thread (no lookahead)
            let stream = self.pa.open_non_blocking_stream(output_settings, move |args| {
                // Fast path: if stream inactive, just zero buffer - NO locks, NO processing
                if !stream_active.load(Ordering::Relaxed) {
                    for sample in args.buffer.iter_mut() {
                        *sample = 0.0;
                    }
                    return pa::Continue;
                }

                // Handle lock failure gracefully - output silence instead of panicking
                let batch = match render_manager.lock() {
                    Ok(mut rm) => {
                        // Check for VisReady or timeout before rendering
                        rm.check_vis_ready();

                        rm.read(
                            Settings::global().buffer_size,
                            Offset {
                                freq: 1.0,
                                gain: 1.0,
                            },
                        )
                    },
                    Err(e) => {
                        eprintln!("ERROR: RenderManager lock poisoned in audio callback: {}", e);
                        None
                    }
                };

                if let Some((b, ramp, _ops)) = batch {
                    new_write_output_buffer(args.buffer, b, ramp);
                    pa::Continue
                } else {
                    write_output_buffer(
                        args.buffer,
                        StereoWaveform::new(Settings::global().buffer_size),
                    );
                    pa::Continue
                }
            })?;

            Ok(stream)
        }
    }

    fn get_output_settings(&self) -> Result<pa::stream::OutputSettings<f32>, Error> {
        let def_output = self.pa.default_output_device()?;
        let output_info = self.pa.device_info(def_output)?;
        let latency = output_info.default_low_output_latency;

        let output_params = pa::StreamParameters::new(
            def_output,
            Settings::global().channels,
            Settings::global().interleaved,
            latency,
        );

        let output_settings = pa::OutputStreamSettings::new(
            output_params,
            Settings::global().sample_rate,
            Settings::global().buffer_size as u32,
        );

        Ok(output_settings)
    }

    /// Create a duplex stream with microphone input for the follow system
    ///
    /// This enables real-time pitch detection from mic input, allowing compositions
    /// to follow/respond to external audio (singing, instruments, etc.)
    pub fn create_duplex_stream(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        f_basis: f64,
    ) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
        let duplex_settings = self.get_duplex_settings()?;
        let buffer_size = Settings::global().buffer_size;

        // Get stream_active flag for fast-path check in callback
        let stream_active = render_manager.lock().unwrap().stream_active();

        let mut input_buffer: RingBuffer<f32> =
            RingBuffer::<f32>::new(Settings::global().yin_buffer_size);

        let stream = self.pa.open_non_blocking_stream(duplex_settings, move |args| {
            // Fast path: if stream inactive, just zero buffer - NO locks, NO processing
            if !stream_active.load(Ordering::Relaxed) {
                for sample in args.out_buffer.iter_mut() {
                    *sample = 0.0;
                }
                return pa::Continue;
            }

            // Capture mic input
            input_buffer.push_vec(args.in_buffer.to_vec());

            // Detect pitch and gain using YIN algorithm
            let mut detection_result: DetectionResult = input_buffer.to_vec().analyze(
                Settings::global().sample_rate as f32,
                Settings::global().probability_threshold,
            );

            let (freq, gain) = Self::process_detection_result(&mut detection_result);

            // Render audio with detected frequency/gain as offset
            // Handle lock failure gracefully - output silence instead of panicking
            let batch = match render_manager.lock() {
                Ok(mut rm) => {
                    // Check for VisReady or timeout before rendering
                    rm.check_vis_ready();

                    rm.read(
                        buffer_size,
                        Offset {
                            freq: freq / f_basis,
                            gain: gain * 0.1,
                        },
                    )
                },
                Err(e) => {
                    eprintln!("ERROR: RenderManager lock poisoned in duplex audio callback: {}", e);
                    None
                }
            };

            if let Some((b, ramp, _ops)) = batch {
                new_write_output_buffer(args.out_buffer, b, ramp);
                pa::Continue
            } else {
                write_output_buffer(
                    args.out_buffer,
                    StereoWaveform::new(Settings::global().buffer_size),
                );
                pa::Continue
            }
        })?;

        Ok(stream)
    }

    fn get_duplex_settings(&self) -> Result<pa::stream::DuplexSettings<f32, f32>, Error> {
        let def_input = self.pa.default_input_device()?;
        let input_params = pa::StreamParameters::<f32>::new(
            def_input,
            1, // Mono input
            Settings::global().interleaved,
            Settings::global().buffer_size as f64 / Settings::global().sample_rate,
        );

        let def_output = self.pa.default_output_device()?;
        let output_params = pa::StreamParameters::new(
            def_output,
            Settings::global().channels, // Stereo output
            Settings::global().interleaved,
            Settings::global().buffer_size as f64 / Settings::global().sample_rate,
        );

        let duplex_settings = pa::DuplexStreamSettings::new(
            input_params,
            output_params,
            Settings::global().sample_rate,
            Settings::global().buffer_size as u32,
        );

        Ok(duplex_settings)
    }

    /// Filter noise from pitch detection results
    ///
    /// Filters out:
    /// - Very quiet sounds (gain < 0.001)
    /// - Frequencies outside human vocal/instrument range (< 60Hz or > 2000Hz)
    fn process_detection_result(result: &mut DetectionResult) -> (f64, f64) {
        if result.gain < 0.001 || result.frequency > 2000.0 || result.frequency < 60.0 {
            result.frequency = 0.0;
            result.gain = 0.0;
        }

        (result.frequency as f64, (result.gain * 10.0) as f64)
    }
}

/// Convenience function to create a PortAudio stream
pub fn create_portaudio_stream(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let backend = PortAudioBackend::new()?;
    let config = BackendConfig {
        mic_input: false,
        use_lookahead: Settings::global().lookahead_buffers > 0,
    };
    backend.create_stream(render_manager, config)
}

/// Convenience function to create a PortAudio duplex stream with mic input
///
/// Enables the follow system to respond to external audio input in real-time.
/// Performs pitch detection on mic input and modulates output based on detected frequency/gain.
///
/// # Arguments
/// * `render_manager` - The render manager to use for audio generation
/// * `f_basis` - Base frequency for normalization (typically the composition's fundamental frequency)
pub fn create_portaudio_duplex_stream(
    render_manager: Arc<Mutex<RenderManager>>,
    f_basis: f64,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let backend = PortAudioBackend::new()?;
    backend.create_duplex_stream(render_manager, f_basis)
}
