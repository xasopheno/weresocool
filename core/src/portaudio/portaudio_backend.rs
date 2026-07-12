/// PortAudio audio backend
///
/// Consolidates the functionality of legacy backend files.
/// Supports both lookahead buffering and direct rendering modes.

use super::backend::{AudioBackend, BackendConfig};
use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_mixed_output_buffer, write_output_buffer},
};
use weresocool_analyze::{Analyze, DetectionResult};
use weresocool_error::Error;
use weresocool_instrument::{Offset, StereoWaveform};
use weresocool_portaudio as pa;
use weresocool_ring_buffer::RingBuffer;
use weresocool_shared::Settings;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

/// One block of captured microphone audio, tagged with the output playhead
/// (absolute frames) at the moment it was captured, plus the YIN detection for
/// that block. The DAW freezes the `(freq,gain)` stream into a layer's events
/// (faithful — it's exactly what drove `Follow`) and keeps `samples` for later
/// re-analysis.
#[derive(Clone, Debug)]
pub struct CapturedBlock {
    pub playhead: u64,
    pub samples: Vec<f32>,
    /// Detected pitch in Hz (0 = no pitch).
    pub freq: f32,
    /// Detected gain (0..1).
    pub gain: f32,
}

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

        // Master clock for DAW cross-stream sync: a monotonic count of frames
        // actually sent to the DAC, advanced below whenever a real buffer is
        // output. Lock-free read by the input-capture stream. Counting output
        // frames (not the render position) makes it lookahead-agnostic.
        let played_frames = render_manager.lock().unwrap().played_frames();
        let buffer_frames = Settings::global().buffer_size as u64;

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
                    played_frames.fetch_add(buffer_frames, Ordering::Relaxed);
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
                    // Advance the DAW master clock (frames sent to DAC).
                    played_frames.fetch_add(buffer_frames, Ordering::Relaxed);
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
        let mic_gain = Arc::new(AtomicU32::new(8.0_f32.to_bits()));
        self.create_duplex_stream_with_capture(render_manager, f_basis, None, mic_gain)
    }

    /// Duplex stream that, in addition to driving the Follow system from the
    /// mic, optionally forwards each block's raw mic frames tagged with the
    /// output playhead to `capture_sender` — letting the DAW loop-record from
    /// the same stream that powers the live monitor.
    pub fn create_duplex_stream_with_capture(
        &self,
        render_manager: Arc<Mutex<RenderManager>>,
        f_basis: f64,
        capture_sender: Option<crossbeam_channel::Sender<CapturedBlock>>,
        // Mic sensitivity (detected gain × this, clamped to 1); live-adjustable
        // via a slider. f32 bits.
        mic_gain: Arc<AtomicU32>,
    ) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
        let duplex_settings = self.get_duplex_settings()?;
        let buffer_size = Settings::global().buffer_size;

        // Get stream_active flag for fast-path check in callback
        let stream_active = render_manager.lock().unwrap().stream_active();

        let mut input_buffer: RingBuffer<f32> =
            RingBuffer::<f32>::new(Settings::global().yin_buffer_size);
        let mut freq_ring = RingBuffer::<f32>::new(3);
        let mut gain_ring = RingBuffer::<f32>::new(3);

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
            // Smooth the pitch/gain stream with a short non-zero average
            // (holds a note through brief confidence dropouts, releases when
            // the ring is mostly silent). Same pattern as mic_to_writer::Processor.
            freq_ring.push(freq as f32);
            gain_ring.push(gain as f32);
            let (freq, gain) = (freq_ring.avg_frequency() as f64, gain_ring.avg_frequency() as f64);

            // Render audio with detected frequency/gain as offset
            // Handle lock failure gracefully - output silence instead of panicking
            let mut playhead = 0u64;
            let batch = match render_manager.lock() {
                Ok(mut rm) => {
                    // Check for VisReady or timeout before rendering
                    rm.check_vis_ready();

                    let mg = f32::from_bits(mic_gain.load(Ordering::Relaxed)) as f64;
                    let b = rm.read(
                        buffer_size,
                        Offset {
                            freq: freq / f_basis,
                            // Map the (often quiet) detected mic gain up by the
                            // live mic-sensitivity knob and clamp. A `Follow`
                            // voice SETS its gain from this; the 4.0 ceiling lets
                            // it go well past unity (louder than the base sound).
                            gain: (gain * mg).min(2.0),
                        },
                    );
                    playhead = rm.samples_processed() as u64;
                    b
                },
                Err(e) => {
                    eprintln!("ERROR: RenderManager lock poisoned in duplex audio callback: {}", e);
                    None
                }
            };

            // Forward raw mic frames + playhead + YIN detection to the DAW
            // recorder. The (freq,gain) here is exactly what drove Follow, so
            // freezing it gives a faithful performance.
            if let Some(tx) = &capture_sender {
                let _ = tx.try_send(CapturedBlock {
                    playhead,
                    samples: args.in_buffer.to_vec(),
                    freq: freq as f32,
                    gain: gain as f32,
                });
            }

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

    /// The DAW two-path mix: the composition + frozen layers pre-render on
    /// `rm_main` (lookahead background thread → smooth, no underruns), while the
    /// live monitor renders DIRECT on `rm_mon` with the mic offset (low latency).
    /// The callback sums the two and forwards captured mic blocks (tagged with
    /// `rm_main`'s playhead — the loop clock) for recording.
    ///
    /// `rm_main` must be init'd with `lookahead_buffers > 0` so its
    /// `BufferManager` exists; `rm_mon` is read directly (its background thread
    /// is never started).
    pub fn create_duplex_mix_stream(
        &self,
        rm_main: Arc<Mutex<RenderManager>>,
        rm_mon: Arc<Mutex<RenderManager>>,
        f_basis: f64,
        capture_sender: Option<crossbeam_channel::Sender<CapturedBlock>>,
        mic_gain: Arc<AtomicU32>,
    ) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
        let duplex_settings = self.get_duplex_settings()?;
        let buffer_size = Settings::global().buffer_size;

        // rm_main gates the stream and pre-renders in the background (lookahead).
        // The callback pops its queue LOCK-FREE (crossbeam receiver) — it must
        // never contend with the background thread's long-held rm_main lock, or
        // the audio deadline gets missed (clicks). check_vis_ready runs on the
        // app's Update loop, not here.
        let stream_active = rm_main.lock().unwrap().stream_active();
        let played_frames = rm_main.lock().unwrap().played_frames();
        // Follow voices in the MAIN comp track the mic too: the callback
        // publishes each block's analysis here and the background renderer
        // applies it to upcoming slices (lookahead-lagged, ~90ms — the tight
        // path is still the direct-rendered monitor below).
        let main_live_offset = rm_main.lock().unwrap().live_offset_handle();
        let buffer_frames = buffer_size as u64;
        let buffer_rx = rm_main
            .lock()
            .unwrap()
            .buffer_receiver()
            .ok_or_else(|| Error::with_msg("mix stream requires rm_main with lookahead_buffers > 0"))?;
        let _render_thread = RenderManager::start_background_rendering(
            Arc::clone(&rm_main),
            Arc::clone(&stream_active),
        );

        let mut input_buffer: RingBuffer<f32> =
            RingBuffer::<f32>::new(Settings::global().yin_buffer_size);
        let mut last_playhead: u64 = 0;
        let mut freq_ring = RingBuffer::<f32>::new(3);
        let mut gain_ring = RingBuffer::<f32>::new(3);

        let stream = self.pa.open_non_blocking_stream(duplex_settings, move |args| {
            if !stream_active.load(Ordering::Relaxed) {
                for sample in args.out_buffer.iter_mut() {
                    *sample = 0.0;
                }
                return pa::Continue;
            }

            // Mic → YIN → offset (drives the live monitor + is captured verbatim).
            input_buffer.push_vec(args.in_buffer.to_vec());
            let mut detection_result: DetectionResult = input_buffer.to_vec().analyze(
                Settings::global().sample_rate as f32,
                Settings::global().probability_threshold,
            );
            let (freq, gain) = Self::process_detection_result(&mut detection_result);
            // Smooth the pitch/gain stream with a short non-zero average
            // (holds a note through brief confidence dropouts, releases when
            // the ring is mostly silent). Same pattern as mic_to_writer::Processor.
            freq_ring.push(freq as f32);
            gain_ring.push(gain as f32);
            let (freq, gain) = (freq_ring.avg_frequency() as f64, gain_ring.avg_frequency() as f64);

            // Path A — the pre-rendered comp + frozen layers. Lock-free pop; the
            // buffer carries its composition position (stamped at render time),
            // which IS the loop clock the DAC is playing right now — the capture
            // playhead frozen layers align to. Falls back to the last known
            // position while starved/paused.
            let (main_part, playhead) = match buffer_rx.try_recv() {
                Ok(p) => {
                    let pos = p.position as u64;
                    last_playhead = pos;
                    (Some((p.waveform, p.ramp)), pos)
                }
                Err(_) => (None, last_playhead),
            };

            // DAC master clock (monotonic frames-to-device).
            played_frames.fetch_add(buffer_frames, Ordering::Relaxed);

            // Path B — the live monitor, rendered direct with the mic offset.
            // rm_mon is tiny (the armed sound only) and has NO background
            // thread, so this lock is uncontended.
            let mon_part = match rm_mon.lock() {
                Ok(mut rm) => {
                    let mg = f32::from_bits(mic_gain.load(Ordering::Relaxed)) as f64;
                    let offset = Offset { freq: freq / f_basis, gain: (gain * mg).min(2.0) };
                    // Same offset feeds rm_main's background renderer.
                    main_live_offset.set(offset.freq, offset.gain);
                    rm.read(buffer_size, offset).map(|(w, r, _)| (w, r))
                }
                Err(e) => {
                    eprintln!("ERROR: rm_mon lock poisoned in mix callback: {}", e);
                    None
                }
            };

            if let Some(tx) = &capture_sender {
                let _ = tx.try_send(CapturedBlock {
                    playhead,
                    samples: args.in_buffer.to_vec(),
                    freq: freq as f32,
                    gain: gain as f32,
                });
            }

            write_mixed_output_buffer(args.out_buffer, [main_part, mon_part]);
            pa::Continue
        })?;

        Ok(stream)
    }

    /// Open an input-only capture stream. Each block snapshots the master
    /// clock (`played_frames`) and ships the mono mic frames tagged with that
    /// playhead over `sender`. Used by the DAW loop recorder to align captured
    /// audio to the composition loop without a duplex stream. `try_send` drops
    /// on a full channel rather than block the audio thread.
    pub fn create_input_capture_stream(
        &self,
        played_frames: Arc<AtomicU64>,
        sender: crossbeam_channel::Sender<CapturedBlock>,
    ) -> Result<pa::Stream<pa::NonBlocking, pa::Input<f32>>, Error> {
        let settings = self.get_input_settings()?;
        let stream = self.pa.open_non_blocking_stream(settings, move |args| {
            let playhead = played_frames.load(Ordering::Relaxed);
            let _ = sender.try_send(CapturedBlock {
                playhead,
                samples: args.buffer.to_vec(),
                freq: 0.0,
                gain: 0.0,
            });
            pa::Continue
        })?;
        Ok(stream)
    }

    fn get_input_settings(&self) -> Result<pa::stream::InputSettings<f32>, Error> {
        let def_input = self.pa.default_input_device()?;
        let input_info = self.pa.device_info(def_input)?;
        let latency = input_info.default_low_input_latency;

        let input_params = pa::StreamParameters::<f32>::new(
            def_input,
            1, // Mono input
            Settings::global().interleaved,
            latency,
        );

        let input_settings = pa::InputStreamSettings::new(
            input_params,
            Settings::global().sample_rate,
            Settings::global().buffer_size as u32,
        );

        Ok(input_settings)
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
    /// Minimum YIN confidence (`1 - normalized_difference`) to sound a note.
    /// The detector's own `probability_threshold` only requires the difference to
    /// dip *below* it, so any accepted pitch has probability > 0.8 — including
    /// the marginal ones. At a note's onset/offset the ring buffer is half
    /// silence, so YIN finds a barely-confident LOW-frequency period and (before
    /// this gate) sounded it as a spurious low note. A sustained tone sits far
    /// under threshold (probability ≈ 0.95+), so requiring high confidence drops
    /// the transient garbage without swallowing real notes.
    const MIN_PITCH_CONFIDENCE: f32 = 0.90;

    fn process_detection_result(result: &mut DetectionResult) -> (f64, f64) {
        if result.gain < 0.001
            || result.frequency > 2000.0
            || result.frequency < 60.0
            || result.probability < Self::MIN_PITCH_CONFIDENCE
        {
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

/// Duplex stream (mic → Follow live monitor) that also forwards raw mic frames
/// to `capture_sender` for the DAW loop recorder. One stream powers both.
pub fn create_portaudio_duplex_stream_with_capture(
    render_manager: Arc<Mutex<RenderManager>>,
    f_basis: f64,
    capture_sender: crossbeam_channel::Sender<CapturedBlock>,
    mic_gain: Arc<AtomicU32>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let backend = PortAudioBackend::new()?;
    backend.create_duplex_stream_with_capture(
        render_manager,
        f_basis,
        Some(capture_sender),
        mic_gain,
    )
}

/// The DAW two-path mix stream: `rm_main` (comp + frozen layers, lookahead) +
/// `rm_mon` (live monitor, direct), summed; forwards mic capture. See
/// `PortAudioBackend::create_duplex_mix_stream`.
pub fn create_portaudio_duplex_mix_stream(
    rm_main: Arc<Mutex<RenderManager>>,
    rm_mon: Arc<Mutex<RenderManager>>,
    f_basis: f64,
    capture_sender: crossbeam_channel::Sender<CapturedBlock>,
    mic_gain: Arc<AtomicU32>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let backend = PortAudioBackend::new()?;
    backend.create_duplex_mix_stream(rm_main, rm_mon, f_basis, Some(capture_sender), mic_gain)
}

/// Play a mono f32 buffer through the default output device, blocking until it
/// finishes. Opens its own short-lived stream (mono duplicated to stereo), so
/// it can coexist with the main render stream. Intended to be called on a
/// background thread (it sleeps for the buffer's duration). Used by the DAW to
/// audition takes.
pub fn play_samples_blocking(samples: Vec<f32>, sample_rate: f64) -> Result<(), Error> {
    if samples.is_empty() {
        return Ok(());
    }
    let pa = pa::PortAudio::new()?;
    let def_output = pa.default_output_device()?;
    let info = pa.device_info(def_output)?;
    let params =
        pa::StreamParameters::<f32>::new(def_output, 2, true, info.default_low_output_latency);
    let settings = pa::OutputStreamSettings::new(params, sample_rate, 256);

    let samples = Arc::new(samples);
    let cb_samples = Arc::clone(&samples);
    let pos = Arc::new(AtomicUsize::new(0));
    let cb_pos = Arc::clone(&pos);

    let mut stream = pa.open_non_blocking_stream(settings, move |args| {
        let mut p = cb_pos.load(Ordering::Relaxed);
        for frame in args.buffer.chunks_mut(2) {
            let v = cb_samples.get(p).copied().unwrap_or(0.0);
            if let Some(l) = frame.get_mut(0) {
                *l = v;
            }
            if let Some(r) = frame.get_mut(1) {
                *r = v;
            }
            p += 1;
        }
        cb_pos.store(p, Ordering::Relaxed);
        if p >= cb_samples.len() {
            pa::Complete
        } else {
            pa::Continue
        }
    })?;

    stream.start()?;
    let dur = samples.len() as f64 / sample_rate + 0.2;
    std::thread::sleep(std::time::Duration::from_secs_f64(dur));
    let _ = stream.stop();
    Ok(())
}

/// Convenience function to create an input-only capture stream for the DAW.
/// Pair with `create_portaudio_stream` (output-only) — no duplex needed. The
/// master clock comes from `RenderManager::played_frames()`.
pub fn create_portaudio_input_capture_stream(
    played_frames: Arc<AtomicU64>,
    sender: crossbeam_channel::Sender<CapturedBlock>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Input<f32>>, Error> {
    let backend = PortAudioBackend::new()?;
    backend.create_input_capture_stream(played_frames, sender)
}
