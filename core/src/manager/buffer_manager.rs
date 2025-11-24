/// Buffer Manager for WereSoCool
///
/// Manages background rendering thread and pre-rendered audio buffer queue.

use std::sync::{Arc, Mutex};
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
}

/// Trait for types that can be rendered in background thread
///
/// This allows BufferManager to be independent of RenderManager's full implementation
pub trait BackgroundRenderable {
    fn is_paused(&self) -> bool;
    fn render_buffer(&mut self, buffer_size: usize, offset: Offset) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)>;
    fn has_current_render(&self) -> bool;
}

/// Start background rendering thread that pre-renders buffers
///
/// Takes a renderable (typically Arc<Mutex<RenderManager>>) and a sender.
/// Returns the JoinHandle for the rendering thread.
pub fn start_background_rendering<R>(
    renderable: Arc<Mutex<R>>,
    sender: crossbeam_channel::Sender<PrerenderedBuffer>,
) -> std::thread::JoinHandle<()>
where
    R: BackgroundRenderable + Send + 'static,
{
    std::thread::Builder::new()
        .name("weresocool-render".to_string())
        .spawn(move || {
            loop {
                // Try to render the next buffer
                let (should_continue, buffer_result) = match renderable.lock() {
                    Ok(mut rm) => {
                        // Check if we should stop
                        if rm.is_paused() {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            (true, None)
                        } else {
                            // Render a buffer
                            let buffer_size = Settings::global().buffer_size;
                            let result = rm.render_buffer(
                                buffer_size,
                                Offset {
                                    freq: 1.0,
                                    gain: 1.0,
                                },
                            );

                            let should_continue = result.is_some() || rm.has_current_render();
                            (should_continue, result)
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR: RenderManager lock poisoned in render loop: {}", e);
                        break; // Exit thread on lock failure
                    }
                }; // Lock is released here

                // Send buffer WITHOUT holding the lock
                let buffer = if let Some((waveform, ramp, ops)) = buffer_result {
                    PrerenderedBuffer { waveform, ramp, ops }
                } else {
                    // No audio to render - send silence to keep queue full and prevent crackling
                    let buffer_size = Settings::global().buffer_size;
                    PrerenderedBuffer {
                        waveform: StereoWaveform::new(buffer_size),
                        ramp: vec![1.0; buffer_size * 2],
                        ops: vec![],
                    }
                };

                // This will block if the queue is full, which is what we want
                // But we're not holding the lock, so audio thread can still pop
                if sender.send(buffer).is_err() {
                    break; // Channel closed, exit thread
                }

                if !should_continue {
                    // No more data to render
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        })
        .expect("Failed to spawn render thread")
}
