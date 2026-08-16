use crate::manager::audio_engine::AudioEngine;
use crate::manager::buffer_manager::{BackgroundRenderable, BufferManager};
use crate::manager::midi_controller::MidiController;
use crate::manager::volume_controller::VolumeController;
use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType, Interpretable},
};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
// std::time::Instant panics on wasm32; web-time is API-identical (reads
// performance.now() in the browser). pause()/check_vis_ready() run on the
// browser build's audio pump, so this path is wasm-reachable.
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};
use std::{path::PathBuf, sync::mpsc::SendError};
use weresocool_ast::Defs;
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, render_voice::renderables_to_render_voices, RenderOp, render_voice::RenderVoice,
};
use weresocool_instrument::{Offset, StereoWaveform};
use weresocool_shared::Settings;
use crate::events::{Events, RenderEvent, StateEvent};

pub type KillChannel = Option<Sender<bool>>;

// Legacy type alias for backward compatibility
#[deprecated(since = "1.0.48", note = "Use RenderEvent instead")]
pub type VisEvent = RenderEvent;

/// Latest live-analysis offset (mic YIN → freq/gain), published lock-free by
/// an audio callback and read by the background (lookahead) renderer. Lets
/// `Follow` voices in a PRE-RENDERED composition track the mic: response lag
/// is the lookahead depth (~90ms at 32×128), vs ~3ms on a direct-render
/// path — fine for textural following, use a direct manager for tight
/// monitoring. Neutral (1.0, 1.0) until something publishes.
#[derive(Debug)]
pub struct LiveOffset {
    freq_bits: AtomicU64,
    gain_bits: AtomicU64,
}

impl Default for LiveOffset {
    fn default() -> Self {
        Self {
            freq_bits: AtomicU64::new(1.0_f64.to_bits()),
            gain_bits: AtomicU64::new(1.0_f64.to_bits()),
        }
    }
}

impl LiveOffset {
    pub fn set(&self, freq: f64, gain: f64) {
        self.freq_bits.store(freq.to_bits(), Ordering::Relaxed);
        self.gain_bits.store(gain.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> Offset {
        Offset {
            freq: f64::from_bits(self.freq_bits.load(Ordering::Relaxed)),
            gain: f64::from_bits(self.gain_bits.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrerenderedBuffer {
    pub waveform: StereoWaveform,
    pub ramp: Vec<f32>,
    pub ops: Vec<Vec<RenderOp>>,
    /// Composition playhead (samples) at the START of this buffer — stamped at
    /// render time, so a consumer knows exactly which timeline slice it's
    /// outputting (survives seeks/pauses; needs no RenderManager lock).
    pub position: usize,
}

#[derive(Debug)]
pub struct RenderManager {
    pub events: Events,
    pub audio_engine: AudioEngine,
    reverse_channel: Option<crossbeam_channel::Receiver<RenderEvent>>,
    volume_controller: VolumeController,
    _read_idx: usize,
    kill_channel: KillChannel,
    once: bool,
    paused: bool,
    paused_at: Option<Instant>,
    /// How long a `pause()` may wait for `VisReady` before giving up and
    /// playing anyway. One second is right for a CLI that has no renderer to
    /// wait for; a windowed session needs longer, because wgpu pipeline
    /// creation on a cold start costs seconds and the whole point of the
    /// handshake is to wait for exactly that.
    vis_ready_timeout: Duration,
    // MIDI
    midi_controller: Option<MidiController>,
    // Background rendering
    buffer_manager: Option<BufferManager>,
    // Stream activity control - when false, audio callback skips processing
    stream_active: Arc<AtomicBool>,
    // Master clock for cross-stream sync (DAW loop record). The output audio
    // callback stores the absolute playhead (`samples_processed`) here after
    // each render so a separate input-capture stream can tag mic frames with
    // the playhead they were recorded against — without locking this manager.
    played_frames: Arc<AtomicU64>,
    // Latest mic-analysis offset for the background (lookahead) renderer, so
    // Follow voices in the pre-rendered composition track the mic. See
    // `LiveOffset`.
    live_offset: Arc<LiveOffset>,
}
pub struct RenderManagerSettings {
    pub sample_rate: f64,
    pub buffer_size: usize,
}

impl RenderManager {
    pub fn init(
        reverse_channel: Option<crossbeam_channel::Receiver<RenderEvent>>,
        kill_channel: KillChannel,
        once: bool,
        settings: Option<RenderManagerSettings>,
    ) -> Self {
        // Only initialize settings if not already set (allows caller to pre-configure)
        if !cfg!(test) && !Settings::is_initialized() {
            if let Some(s) = settings {
                Settings::init(s.sample_rate, s.buffer_size);
            } else {
                Settings::init_default();
            };
        }

        let lookahead_buffers = Settings::global().lookahead_buffers;
        let buffer_manager = BufferManager::new(lookahead_buffers);

        Self {
            events: Events::new(),
            reverse_channel,
            audio_engine: AudioEngine::new(),
            volume_controller: VolumeController::new(),
            _read_idx: 0,
            kill_channel,
            once,
            paused: false,
            paused_at: None,
            vis_ready_timeout: Duration::from_secs(1),
            midi_controller: crate::manager::midi_controller::MidiClient::new("127.0.0.1:6479")
                .ok()
                .map(MidiController::new),
            buffer_manager,
            stream_active: Arc::new(AtomicBool::new(true)),
            played_frames: Arc::new(AtomicU64::new(0)),
            live_offset: Arc::new(LiveOffset::default()),
        }
    }

    pub fn init_wasm(settings: Option<RenderManagerSettings>) -> Self {
        // Same guard as `init`: a caller that already configured the
        // process-wide Settings (e.g. the AU plugin, which must keep the
        // HOST's sample rate) wins — re-running init here would reload
        // config files over it.
        if !cfg!(test) && !Settings::is_initialized() {
            if let Some(s) = settings {
                Settings::init(s.sample_rate, s.buffer_size);
            } else {
                Settings::init_default();
            };
        }

        let lookahead_buffers = Settings::global().lookahead_buffers;
        let buffer_manager = BufferManager::new(lookahead_buffers);

        Self {
            events: Events::new(),
            reverse_channel: None,
            audio_engine: AudioEngine::new(),
            volume_controller: VolumeController::new(),
            _read_idx: 0,
            kill_channel: None,
            once: false,
            paused: false,
            paused_at: None,
            vis_ready_timeout: Duration::from_secs(1),
            midi_controller: None,
            buffer_manager,
            stream_active: Arc::new(AtomicBool::new(true)),
            played_frames: Arc::new(AtomicU64::new(0)),
            live_offset: Arc::new(LiveOffset::default()),
        }
    }

    pub fn kill(&mut self) -> Result<(), SendError<bool>> {
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Stopped);
        }
        if let Some(kc) = &self.kill_channel {
            kc.send(true)?;
            #[cfg(target_os = "linux")]
            std::thread::sleep(std::time::Duration::from_millis(500));
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn play(&mut self) {
        self.paused = false;
        self.paused_at = None;
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Paused(false));
        }
    }

    /// How long `pause()` waits for `VisReady` before playing anyway.
    pub fn set_vis_ready_timeout(&mut self, timeout: Duration) {
        self.vis_ready_timeout = timeout;
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.paused_at = Some(Instant::now());
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Paused(true));
        }
    }

    /// Get the stream_active flag for use in audio callback
    /// When false, the audio callback should skip all processing
    pub fn stream_active(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stream_active)
    }

    /// Set the stream active state
    pub fn set_stream_active(&self, active: bool) {
        self.stream_active.store(active, Ordering::SeqCst);
    }

    /// Master-clock handle for DAW cross-stream sync: a monotonic count of
    /// frames sent to the DAC, advanced by the output audio callback. Read
    /// lock-free by the input-capture stream to align recorded mic frames to
    /// the composition loop (loop position = `played_frames % total_samples`).
    pub fn played_frames(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.played_frames)
    }

    /// Handle for publishing the latest mic-analysis offset. An audio
    /// callback that analyzes live input stores `(freq/f_basis, gain)` here;
    /// the background (lookahead) renderer picks it up per slice so `Follow`
    /// voices in the pre-rendered composition track the mic.
    pub fn live_offset_handle(&self) -> Arc<LiveOffset> {
        Arc::clone(&self.live_offset)
    }

    /// Check for VisReady event and unpause if received (non-blocking)
    /// Also enforces a 1-second timeout - if paused for too long, automatically unpause
    pub fn check_vis_ready(&mut self) -> bool {
        // Check for timeout - auto-unpause after 1 second if VisReady never arrives
        if let Some(paused_time) = self.paused_at {
            if paused_time.elapsed() > self.vis_ready_timeout {
                self.play();
                return true; // Indicate unpause happened (via timeout)
            }
        }

        // Check for VisReady event from visualization
        if let Some(rx) = &self.reverse_channel {
            if let Ok(event) = rx.try_recv() {
                if matches!(event, RenderEvent::VisReady) {
                    self.play();
                    return true; // Indicate unpause happened (via VisReady)
                }
            }
        }
        false
    }

    pub fn update_volume(&mut self, volume: f32) {
        self.volume_controller.update_volume(volume, &mut self.events);
    }

    pub fn current_volume(&self) -> f32 {
        self.volume_controller.current_volume()
    }

    pub fn past_volume(&self) -> f32 {
        self.volume_controller.past_volume()
    }

    fn ramp_to_current_volume(&mut self, buffer_size: usize) -> Vec<f32> {
        self.volume_controller.ramp_to_current_volume(buffer_size)
    }

    /// Pop a pre-rendered buffer from the queue (non-blocking, for audio thread)
    pub fn pop_buffer(&self) -> Option<PrerenderedBuffer> {
        self.buffer_manager.as_ref()?.pop_buffer()
    }

    /// Drain all buffers from the queue (use when switching renders to clear stale audio)
    pub fn drain_buffer_queue(&self) {
        if let Some(manager) = &self.buffer_manager {
            manager.drain_buffer_queue();
        }
    }

    /// Lock-free pop handle for the pre-rendered buffer queue. An audio
    /// callback holding this never touches the RenderManager mutex (which the
    /// background render thread holds for whole render slices).
    pub fn buffer_receiver(&self) -> Option<crossbeam_channel::Receiver<PrerenderedBuffer>> {
        self.buffer_manager.as_ref().and_then(|bm| bm.receiver())
    }

    /// Start background rendering thread that pre-renders buffers
    /// Returns the JoinHandle for the rendering thread
    pub fn start_background_rendering(render_manager: Arc<Mutex<RenderManager>>, stream_active: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
        // Get the sender once at the start, outside the loop
        let sender = match render_manager.lock() {
            Ok(rm) => rm.buffer_manager.as_ref().and_then(|bm| bm.sender()),
            Err(e) => {
                eprintln!("ERROR: RenderManager lock poisoned at thread start: {}", e);
                None
            }
        };

        let Some(sender) = sender else {
            // No buffer manager or sender, return a dummy thread that exits immediately
            return std::thread::Builder::new()
                .name("weresocool-render-dummy".to_string())
                .spawn(|| {})
                .expect("Failed to spawn dummy thread");
        };

        crate::manager::buffer_manager::start_background_rendering(render_manager, sender, stream_active)
    }

    pub fn read(
        &mut self,
        buffer_size: usize,
        offset: Offset,
    ) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)> {
        if self.paused {
            return None;
        }

        let has_render_subscribers = self.events.render.has_subscribers();

        // Delegate to AudioEngine for rendering
        let result = match self.audio_engine.render(buffer_size, offset, has_render_subscribers) {
            Some(result) => result,
            None => {
                // Composition exhausted: in "once" mode this is the completion
                // signal. An early `?` return here used to skip the kill check
                // below, leaving `play` blocked on its kill channel forever.
                // Ignore the send result: the receiver is gone once the main
                // thread has already begun exiting.
                if self.once && !self.audio_engine.exists_next_render() {
                    let _ = self.kill();
                }
                return None;
            }
        };

        // Handle "once" mode - kill if rendering is complete
        if self.once && !self.audio_engine.exists_current_render() && !self.audio_engine.exists_next_render() {
            self.kill().expect("Unable to kill");
        }

        // Send MIDI events if controller is available
        if let Some(controller) = &self.midi_controller {
            controller.send_midi_events(&result.midi_ops, result.read_start_samples, &mut self.events);
        }

        // Emit visualization events if there are subscribers
        if has_render_subscribers {
            let ops = result.ops_per_voice.iter().flatten().cloned().collect::<Vec<_>>();
            let normalizer = self.audio_engine.normalizer();
            let opmap = crate::manager::visualization_adapter::VisualizationAdapter::render_ops_to_opmap(&ops, normalizer);
            self.events.render.emit(RenderEvent::Ops(opmap));
        }

        let ramp = self.ramp_to_current_volume(buffer_size);
        Some((result.waveform, ramp, result.ops_per_voice))
    }

    pub fn inc_render(&mut self, copy_oscillators: bool) {
        // Delegate to AudioEngine for render switching
        self.audio_engine.inc_render(copy_oscillators);

        // Send visualization reset and audio ready events (non-blocking to prevent audio glitches)
        if self.events.render.has_subscribers() {
            self.events.render.emit(RenderEvent::Reset);
            self.events.render.emit(RenderEvent::AudioReady);

            // Only pause if there's a reverse channel (external visualizer that can respond)
            // This prevents hanging CLI tools that subscribe to events but can't send VisReady
            if self.reverse_channel.is_some() {
                self.pause();
            }
        }
    }

    /// Seek the playback head to `target_sample` (absolute, from the
    /// start of the current render). Used by scrubbing — the bevy
    /// slider in kintaro calls this when the user releases a drag.
    ///
    /// Side effects:
    ///   1. All voices reset oscillator state and reposition their cursors.
    ///   2. `AudioEngine::samples_processed` jumps to `target_sample`.
    ///   3. The post-seek gain ramp arms (5 ms fade-in masks the click).
    ///   4. The pre-rendered buffer queue drains, so audio from BEFORE
    ///      the seek doesn't keep playing for the lookahead window.
    ///
    /// What this does NOT touch:
    ///   - Subscribers ARE notified via `RenderEvent::Reset` so visual
    ///     pipelines (kintaro's brush pool, warp ping-pong) can wipe.
    ///   - We do not pause: the background renderer will refill the
    ///     queue from the new position on its next tick.
    pub fn seek_to_sample(&mut self, target_sample: usize) {
        self.audio_engine.seek_to_sample(target_sample);
        self.drain_buffer_queue();
        // Tell any subscribers (kintaro's brush pool, warp feedback) to
        // wipe their accumulated state. Same channel `inc_render` uses
        // for the same reason: state that depends on past frames is no
        // longer valid past a discontinuity.
        if self.events.render.has_subscribers() {
            self.events.render.emit(RenderEvent::Reset);
        }
    }

    /// Set (or clear with `None`) the sub-loop window `[start, end)` in samples.
    /// Playback then cycles within it, silently (no `Reset` — the visualizer
    /// keeps its state). NOT drained: a live handle-drag calls this every frame,
    /// and draining each time would stutter; the new region just takes effect
    /// within a lookahead's worth of already-buffered audio.
    pub fn set_loop_region(&mut self, region: Option<(usize, usize)>) {
        self.audio_engine.set_loop_region(region);
    }

    /// Current playhead in samples. Exposed for the scrub UI so the
    /// slider can track the live audio position between user drags.
    pub fn samples_processed(&self) -> usize {
        self.audio_engine.samples_processed()
    }

    /// Total samples in the currently-loaded render. Returns `None`
    /// when no render is loaded yet (UI should hide the slider in
    /// that case rather than showing 0).
    pub fn total_samples(&self) -> Option<usize> {
        self.audio_engine.total_samples()
    }

    pub fn push_render(&mut self, render: Vec<RenderVoice>, once: bool) {
        self.once = once;
        self.set_stream_active(true);  // Activate stream when new render arrives
        // Clear any pre-rendered buffers from old render to avoid latency
        self.drain_buffer_queue();
        self.audio_engine.push_render(render);
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Started);
        }
    }

    /// Set the runtime output gain on every current-render voice tagged with
    /// `tag` (a `#tag` marker in the source). Live per-voice fader for a DAW
    /// mixer — no re-render, no events. Returns the number of voices matched.
    pub fn set_gain_for_tagged_voices(&mut self, tag: &str, gain: f64) -> usize {
        self.audio_engine.set_gain_for_tagged_voices(tag, gain)
    }

    /// Hot-swap the current render's voices in place, preserving the playhead
    /// and emitting NO render events. See `AudioEngine::swap_current_render`.
    ///
    /// This is the live-mix-change path: a DAW mute/solo/volume/arm tweak
    /// re-renders the composition with the new mix and swaps it in without a
    /// timeline `Reset`, so a subscribed visualizer never wipes its accumulated
    /// state and playback never jumps back to the start. Do NOT drain the
    /// buffer queue — the already-buffered old-mix audio plays out seamlessly
    /// into the swapped voices.
    pub fn swap_current_render(&mut self, render: Vec<RenderVoice>) {
        self.set_stream_active(true);
        self.audio_engine.swap_current_render(render);
    }
}

impl BackgroundRenderable for RenderManager {
    fn is_paused(&self) -> bool {
        self.paused
    }

    fn poll_vis_ready(&mut self) {
        self.check_vis_ready();
    }

    fn background_offset(&self) -> Offset {
        self.live_offset.get()
    }

    fn render_buffer(&mut self, buffer_size: usize, offset: Offset) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)> {
        self.read(buffer_size, offset)
    }

    fn has_current_render(&self) -> bool {
        self.audio_engine.exists_current_render()
    }

    fn position(&self) -> usize {
        self.audio_engine.samples_processed()
    }
}

pub fn prepare_render_outside(
    input: InputType<'_>,
    working_path: Option<PathBuf>,
) -> Result<(Vec<RenderVoice>, Defs), Error> {
    let (nf, basis, mut table) = match input.make(RenderType::NfBasisAndTable, working_path)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => return Err(Error::with_msg("Failed Parse/Render")),
    };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;

    let render_voices = renderables_to_render_voices(renderables);

    Ok((render_voices, table))
}

#[cfg(test)]
mod render_manager_tests {
    use super::*;
    use weresocool_instrument::renderable::RenderOp;
    use weresocool_shared::helpers::{cmp_f32, cmp_vec_f32};

    #[test]
    fn test_ramp_to_current_value() {
        Settings::init_test();
        let mut rm = RenderManager::init(None, None, false, None);
        rm.update_volume(0.9);
        assert!(cmp_f32(rm.current_volume(), f32::powf(0.9, 2.0)));
        let ramp = rm.ramp_to_current_volume(2);
        dbg!(&ramp);
        assert!(cmp_vec_f32(
            ramp,
            vec![0.8, 0.8025, 0.80499995, 0.807_499_95]
        ));
    }

    #[test]
    fn test_inc_render() {
        Settings::init_test();
        let mut r = RenderManager::init(None, None, false, None);
        // Push a render to next slot
        r.push_render(render_voices_mock(), false);
        assert!(r.audio_engine.exists_next_render());
        // Increment should move it to current
        r.inc_render(true);
        assert!(r.audio_engine.exists_current_render());
        // Increment again with nothing in next should clear current
        r.inc_render(true);
        assert!(!r.audio_engine.exists_current_render());
    }

    fn render_voices_mock() -> Vec<RenderVoice> {
        vec![RenderVoice::init(&[RenderOp::init_silent_with_length(1.0)])]
    }

    #[test]
    fn test_check_vis_ready_timeout() {
        use std::thread;
        use std::time::Duration;

        Settings::init_test();
        let mut rm = RenderManager::init(None, None, false, None);

        // Manually pause to simulate inc_render pause
        rm.pause();
        assert!(rm.paused);
        assert!(rm.paused_at.is_some());

        // check_vis_ready should not unpause immediately
        let unpause_immediate = rm.check_vis_ready();
        assert!(!unpause_immediate);
        assert!(rm.paused);

        // Wait for timeout (1 second + buffer)
        thread::sleep(Duration::from_millis(1100));

        // check_vis_ready should now timeout and unpause
        let unpause_after_timeout = rm.check_vis_ready();
        assert!(unpause_after_timeout);
        assert!(!rm.paused);
        assert!(rm.paused_at.is_none());
    }

    #[test]
    fn test_inc_render_without_reverse_channel() {
        Settings::init_test();

        // Create RenderManager without reverse_channel (like CLI tools)
        let mut rm = RenderManager::init(None, None, false, None);

        // Subscribe to render events (simulating a listener)
        let _event_rx = rm.events.render.subscribe();

        // inc_render should NOT pause when there's no reverse_channel
        rm.inc_render(true);
        assert!(!rm.paused);
        assert!(rm.paused_at.is_none());

        // Create RenderManager WITH reverse_channel (external visualizer)
        let (_reverse_tx, reverse_rx) = crossbeam_channel::unbounded();
        let mut rm_with_channel = RenderManager::init(Some(reverse_rx), None, false, None);

        // Subscribe to render events
        let _event_rx2 = rm_with_channel.events.render.subscribe();

        // inc_render SHOULD pause when there's a reverse_channel
        rm_with_channel.inc_render(true);
        assert!(rm_with_channel.paused);
        assert!(rm_with_channel.paused_at.is_some());
    }

    #[test]
    fn test_push_render() {
        Settings::init_test();
        let mut r = RenderManager::init(None, None, false, None);
        assert_eq!(*r.audio_engine.current_render(), None);
        assert_eq!(*r.audio_engine.next_render(), None);
        r.push_render(render_voices_mock(), false);
        assert_eq!(*r.audio_engine.next_render(), Some(render_voices_mock()));
        assert_eq!(*r.audio_engine.current_render(), None);
        r.push_render(render_voices_mock(), false);
    }
}
