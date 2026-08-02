/// Buffer Manager for WereSoCool
///
/// Manages background rendering thread and pre-rendered audio buffer queue.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use weresocool_instrument::{Offset, StereoWaveform, RenderOp};
use weresocool_shared::Settings;

use super::render_manager::PrerenderedBuffer;

#[derive(Debug)]
pub struct BufferManager {
    sender: Option<crossbeam_channel::Sender<PrerenderedBuffer>>,
    receiver: Option<crossbeam_channel::Receiver<PrerenderedBuffer>>,
}

impl BufferManager {
    /// Create a new BufferManager with the specified number of lookahead buffers
    ///
    /// If lookahead_buffers is 0, returns None (no buffering).
    pub fn new(lookahead_buffers: usize) -> Option<Self> {
        if lookahead_buffers > 0 {
            let (sender, receiver) = crossbeam_channel::bounded(lookahead_buffers);
            Some(Self {
                sender: Some(sender),
                receiver: Some(receiver),
            })
        } else {
            None
        }
    }

    /// Pop a pre-rendered buffer from the queue (non-blocking, for audio thread)
    pub fn pop_buffer(&self) -> Option<PrerenderedBuffer> {
        if let Some(receiver) = &self.receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    /// Drain all buffers from the queue (use when switching renders to clear stale audio)
    pub fn drain_buffer_queue(&self) {
        if let Some(receiver) = &self.receiver {
            while receiver.try_recv().is_ok() {
                // Discard all buffers
            }
        }
    }

    /// Get a clone of the sender for use in background thread
    pub fn sender(&self) -> Option<crossbeam_channel::Sender<PrerenderedBuffer>> {
        self.sender.clone()
    }

    /// Clone of the receiver — lets an audio callback pop pre-rendered buffers
    /// WITHOUT locking the RenderManager (crossbeam channels are mpmc). The
    /// audio thread must never contend with the background render thread's
    /// long-held RenderManager lock.
    pub fn receiver(&self) -> Option<crossbeam_channel::Receiver<PrerenderedBuffer>> {
        self.receiver.clone()
    }
}

/// Trait for types that can be rendered in background thread
///
/// This allows BufferManager to be independent of RenderManager's full implementation
pub trait BackgroundRenderable {
    fn is_paused(&self) -> bool;
    /// Poll for the visualizer's readiness (or its timeout) and unpause.
    /// Lives on the render thread because it needs the RenderManager lock,
    /// which the audio callback must never take. Default: nothing to poll.
    fn poll_vis_ready(&mut self) {}
    fn render_buffer(&mut self, buffer_size: usize, offset: Offset) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)>;
    fn has_current_render(&self) -> bool;
    /// Composition playhead (samples) — read AFTER render_buffer to stamp the
    /// rendered slice's end position.
    fn position(&self) -> usize;
    /// Offset to render background slices with. Neutral by default; a
    /// manager whose live-input callback publishes mic analysis (see
    /// `RenderManager::live_offset_handle`) overrides this so `Follow`
    /// voices in the pre-rendered composition still track the mic.
    fn background_offset(&self) -> Offset {
        Offset { freq: 1.0, gain: 1.0 }
    }
}

/// Start background rendering thread that pre-renders buffers
///
/// Takes a renderable (typically Arc<Mutex<RenderManager>>), a sender, and stream_active flag.
/// Returns the JoinHandle for the rendering thread.
pub fn start_background_rendering<R>(
    renderable: Arc<Mutex<R>>,
    sender: crossbeam_channel::Sender<PrerenderedBuffer>,
    stream_active: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()>
where
    R: BackgroundRenderable + Send + 'static,
{
    std::thread::Builder::new()
        .name("weresocool-render".to_string())
        .spawn(move || {
            loop {
                // Fast path: if stream is inactive, sleep long and skip everything
                if !stream_active.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    continue;
                }

                // Try to render the next buffer
                let (should_continue, buffer_result, is_paused, position) = match renderable.lock() {
                    Ok(mut rm) => {
                        rm.poll_vis_ready();
                        let paused = rm.is_paused();
                        if paused {
                            // When paused, don't render - just check state periodically
                            (true, None, true, rm.position())
                        } else {
                            // Render a buffer with the latest live offset
                            // (neutral unless a mic callback publishes —
                            // this is what lets Follow voices in the MAIN
                            // composition track the mic, at lookahead lag).
                            let buffer_size = Settings::global().buffer_size;
                            let offset = rm.background_offset();
                            let result = rm.render_buffer(buffer_size, offset);

                            let should_continue = result.is_some() || rm.has_current_render();
                            // Playhead AFTER the render = end of this slice; the
                            // buffer's start = end - buffer_size.
                            let end = rm.position();
                            let start = end.saturating_sub(buffer_size);
                            (should_continue, result, false, start)
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR: RenderManager lock poisoned in render loop: {}", e);
                        break; // Exit thread on lock failure
                    }
                }; // Lock is released here

                // When paused, sleep long and don't send buffers - nothing needs them
                if is_paused {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }

                // Send buffer WITHOUT holding the lock
                let had_audio = buffer_result.is_some();
                let buffer = if let Some((waveform, ramp, ops)) = buffer_result {
                    PrerenderedBuffer { waveform, ramp, ops, position }
                } else {
                    // No audio to render - send silence to keep queue full and prevent crackling
                    let buffer_size = Settings::global().buffer_size;
                    PrerenderedBuffer {
                        waveform: StereoWaveform::new(buffer_size),
                        ramp: vec![1.0; buffer_size * 2],
                        ops: vec![],
                        position,
                    }
                };

                // This will block if the queue is full, which is what we want
                // But we're not holding the lock, so audio thread can still pop
                if sender.send(buffer).is_err() {
                    break; // Channel closed, exit thread
                }

                // Deactivate stream when render is finished (no more audio to produce)
                if !should_continue && !had_audio {
                    stream_active.store(false, Ordering::SeqCst);
                }

                // Sleep when no real audio was rendered to prevent busy-looping
                if !should_continue || !had_audio {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        })
        .expect("Failed to spawn render thread")
}
