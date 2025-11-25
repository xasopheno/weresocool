use crate::manager::audio_engine::AudioEngine;
use crate::manager::buffer_manager::{BackgroundRenderable, BufferManager};
use crate::manager::midi_controller::MidiController;
use crate::manager::volume_controller::VolumeController;
use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType, Interpretable},
};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::time::{Duration, Instant};
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

#[derive(Debug, Clone)]
pub struct PrerenderedBuffer {
    pub waveform: StereoWaveform,
    pub ramp: Vec<f32>,
    pub ops: Vec<Vec<RenderOp>>,
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
    // MIDI
    midi_controller: Option<MidiController>,
    // Background rendering
    buffer_manager: Option<BufferManager>,
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
        if !cfg!(test) {
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
            midi_controller: crate::manager::midi_controller::MidiClient::new("127.0.0.1:6479")
                .ok()
                .map(MidiController::new),
            buffer_manager,
        }
    }

    pub fn init_wasm(settings: Option<RenderManagerSettings>) -> Self {
        if !cfg!(test) {
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
            midi_controller: None,
            buffer_manager,
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

    pub fn pause(&mut self) {
        self.paused = true;
        self.paused_at = Some(Instant::now());
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Paused(true));
        }
    }

    /// Check for VisReady event and unpause if received (non-blocking)
    /// Also enforces a 1-second timeout - if paused for too long, automatically unpause
    pub fn check_vis_ready(&mut self) -> bool {
        // Check for timeout - auto-unpause after 1 second if VisReady never arrives
        if let Some(paused_time) = self.paused_at {
            if paused_time.elapsed() > Duration::from_secs(1) {
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

    /// Start background rendering thread that pre-renders buffers
    /// Returns the JoinHandle for the rendering thread
    pub fn start_background_rendering(render_manager: Arc<Mutex<RenderManager>>) -> std::thread::JoinHandle<()> {
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

        crate::manager::buffer_manager::start_background_rendering(render_manager, sender)
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
        let result = self.audio_engine.render(buffer_size, offset, has_render_subscribers)?;

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

    pub fn push_render(&mut self, render: Vec<RenderVoice>, once: bool) {
        self.once = once;
        self.audio_engine.push_render(render);
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Started);
        }
    }
}

impl BackgroundRenderable for RenderManager {
    fn is_paused(&self) -> bool {
        self.paused
    }

    fn render_buffer(&mut self, buffer_size: usize, offset: Offset) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)> {
        self.read(buffer_size, offset)
    }

    fn has_current_render(&self) -> bool {
        self.audio_engine.exists_current_render()
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
