use crate::generation::Normalizer;
use crate::manager::buffer_manager::{BackgroundRenderable, BufferManager};
use crate::manager::midi_controller::MidiController;
use crate::manager::resizeable_2d_vec::Resizeable2DVec;
use crate::manager::volume_controller::VolumeController;
use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    generation::sum_all_waveforms,
    generation::Op4D,
    interpretable::{InputType, Interpretable},
};
use opmap::OpMap;
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::{path::PathBuf, sync::mpsc::SendError};
use weresocool_ast::{Defs};
use weresocool_ast::follow::evaluate::EvaluateAction;
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, render_voice::renderables_to_render_voices, RenderOp, render_voice::RenderVoice, Renderable,
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
    normalizer: Normalizer,
    reverse_channel: Option<crossbeam_channel::Receiver<RenderEvent>>,
    pub renders: [Option<Vec<RenderVoice>>; 2],
    volume_controller: VolumeController,
    render_idx: usize,
    _read_idx: usize,
    kill_channel: KillChannel,
    once: bool,
    paused: bool,
    samples_processed: usize,
    // MIDI
    midi_controller: Option<MidiController>,
    // Background rendering
    buffer_manager: Option<BufferManager>,
}


pub fn render_op_to_normalized_op4d_list(
    render_op: &RenderOp,
    normalizer: &Normalizer,
    frame_length: f64,  // The size of each chunk, e.g. 1/30 = 0.0333 for 30 FPS
) -> Vec<Op4D> {
    // If these conditions fail, just return an empty Vec:
    if render_op.f == 0.0 || render_op.g == (0.0, 0.0) {
        return vec![];
    }

    // We'll subdivide render_op.l in multiples of frame_length
    let total_length = render_op.l;
    if total_length <= 0.0 {
        return vec![]; 
    }

    let mut out_ops = Vec::new();

    let mut current_time = render_op.t;
    let mut remaining = total_length;

    while remaining > 0.0 {
        // Take either a full frame_length or whatever leftover remains
        let slice_len = if remaining >= frame_length {
            frame_length
        } else {
            remaining
        };

        // Build a brand-new Op4D for just this slice
        let mut op4d = Op4D {
            y: render_op.f,
            z: (render_op.g.0 + render_op.g.1) / 2.0,
            x: render_op.p,
            l: slice_len,
            t: current_time,
            voice: render_op.voice,
            event: render_op.event,
            names: render_op.names.clone(),
            colors: render_op.colors.clone(),
            wgsl: render_op.wgsl.clone(),
        };

        // Apply your normalization
        op4d.normalize(normalizer);

        out_ops.push(op4d);

        // Advance current_time for the next slice
        current_time += slice_len;
        // Subtract this slice from the remaining length
        remaining -= slice_len;
    }

    out_ops
}

pub fn render_op_to_normalized_op4d(render_op: &RenderOp, normalizer: &Normalizer) -> Option<Op4D> {
    if render_op.f == 0.0 || render_op.g == (0.0, 0.0) {
        return None;
    };

    let mut op4d = Op4D {
        y: render_op.f,
        z: (render_op.g.0 + render_op.g.1) / 2.0,
        x: render_op.p,
        l: render_op.l,
        t: render_op.t,
        voice: render_op.voice,
        event: render_op.event,
        names: render_op.names.to_vec(),
        colors: render_op.colors.to_vec(),
        wgsl: render_op.wgsl.clone(),
    };

    op4d.normalize(normalizer);

    Some(op4d)
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
            normalizer: Normalizer::default(),
            reverse_channel,
            renders: [None, None],
            volume_controller: VolumeController::new(),
            render_idx: 0,
            _read_idx: 0,
            kill_channel,
            once,
            paused: false,
            samples_processed: 0,
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
            normalizer: Normalizer::default(),
            reverse_channel: None,
            renders: [None, None],
            volume_controller: VolumeController::new(),
            render_idx: 0,
            _read_idx: 0,
            kill_channel: None,
            once: false,
            paused: false,
            samples_processed: 0,
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
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Paused(false));
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
        if self.events.state.has_subscribers() {
            self.events.state.emit(StateEvent::Paused(true));
        }
    }

    /// Check for VisReady event and unpause if received (non-blocking)
    pub fn check_vis_ready(&mut self) -> bool {
        if let Some(rx) = &self.reverse_channel {
            if let Ok(event) = rx.try_recv() {
                if matches!(event, RenderEvent::VisReady) {
                    self.play();
                    return true;
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

        let mut remaining_buffer_size = buffer_size;
        // Collect ops for visualization only
        let mut total_ops: Resizeable2DVec<RenderOp> = Resizeable2DVec::new(1);
        // Final combined waveform we build progressively
        let mut combined_sw = StereoWaveform::new_empty();
        // MIDI ops accumulated for this read window
        let mut midi_ops: Vec<RenderOp> = Vec::new();

        let normalizer = self.normalizer;
        let has_render_subscribers = self.events.render.has_subscribers();

        // Track absolute sample position at the start of this read
        let read_start_samples = self.samples_processed;
        while remaining_buffer_size > 0 {
            // Compute next_exists before mutable borrow
            let next_exists = self.exists_next_render();

            // Start mutable borrow scope
            let (samples_processed, render_finished) = {
                let current_render_option = self.current_render();

                match current_render_option {
                    Some(render_voices) => {
                        let mut any_data_rendered = false;
                        let mut rendered_per_voice: Vec<StereoWaveform> = Vec::new();

                        let mut min_samples_processed = remaining_buffer_size;

                        for (i, voice) in render_voices.iter_mut().enumerate() {
                            match voice.get_batch(
                                remaining_buffer_size,
                                None,
                                !next_exists && Settings::global().loop_play,
                            ) {
                                Some(batch) => {
                                    any_data_rendered = true;
                                    let samples = batch.iter().map(|op| op.samples).sum::<usize>();
                                    min_samples_processed = min_samples_processed.min(samples);

                                    // Split MIDI-directed ops from audio-directed
                                    let (midi_batch, mut audio_batch): (Vec<_>, Vec<_>) = batch
                                        .into_iter()
                                        .partition(|op| !op.midi.is_empty());

                                    midi_ops.extend(midi_batch.into_iter());

                                    let voice_rendered =
                                        audio_batch.render(&mut voice.oscillator, Some(&offset));
                                    rendered_per_voice.push(voice_rendered);

                                    if has_render_subscribers {
                                        let vis_threshold = (Settings::global().vis_filter_rate * 100.0) as usize;
                                        let b: Vec<_> = audio_batch
                                            .iter()
                                            .filter(|op| {
                                                let hash = op.index.wrapping_mul(2654435761) % 100;
                                                hash < vis_threshold
                                            })
                                            .cloned()
                                            .map(|mut op| {
                                                let follow_offset = op.follows.eval_value(
                                                    offset.freq as f32,
                                                    offset.gain as f32,
                                                );
                                                op.f *= follow_offset.0 as f64;
                                                op.g = (
                                                    op.g.0 * follow_offset.1 as f64,
                                                    op.g.1 * follow_offset.1 as f64,
                                                );
                                                op
                                            })
                                            .collect();

                                        total_ops.extend_at(i, b);
                                        // ops_per_voice.push(voice_ops);
                                    }
                                }
                                None => {
                                    // Voice has finished
                                }
                            }
                        }

                        if any_data_rendered && min_samples_processed > 0 {
                            // Mix this batch now into the running stereo waveform
                            let batch_sw = sum_all_waveforms(rendered_per_voice);
                            combined_sw.append(batch_sw);
                            // Advance absolute playhead samples
                            self.samples_processed = self.samples_processed.saturating_add(min_samples_processed);
                            (min_samples_processed, false)
                        } else if any_data_rendered {
                            // Some data rendered, but min_samples_processed is zero
                            (0, false)
                        } else {
                            // All voices have finished
                            (0, true)
                        }
                    }
                    None => {
                        // No current render
                        (0, true)
                    }
                }
            }; // End of mutable borrow

            if samples_processed > 0 {
                remaining_buffer_size = remaining_buffer_size.saturating_sub(samples_processed);
            }

            if render_finished || (next_exists && !Settings::global().loop_play) {
                if self.exists_next_render() {
                    self.inc_render(true); // Copy oscillators for seamless transitions
                    continue; // Continue processing with next render
                } else {
                    if self.once {
                        self.kill().expect("Unable to kill");
                    }
                    break; // No more renders, exit loop
                }
            }

            if samples_processed == 0 {
                // No samples processed, break to avoid infinite loop
                break;
            }
        }

        // Send MIDI events if controller is available
        if let Some(controller) = &self.midi_controller {
            controller.send_midi_events(&midi_ops, read_start_samples, &mut self.events);
        }

        // If we rendered anything, pad to the buffer size and return
        if combined_sw.l_buffer.len() > 0 {
            combined_sw.pad(buffer_size);

            // Visualization
            if has_render_subscribers {
                let ops = total_ops.to_vec_flat();
                let mut opmap: OpMap<Op4D> = OpMap::with_capacity(ops.len());
                ops.iter().for_each(|v| {
                    let name = v.colors.last().map_or("nameless", |n| n);

                    let op = render_op_to_normalized_op4d(v, &normalizer);
                    if let Some(o) = op {
                        opmap.insert(name, o);
                    };
                });

                self.events.render.emit(RenderEvent::Ops(opmap));
            }

            let ramp = self.ramp_to_current_volume(buffer_size);
            Some((combined_sw, ramp, total_ops.to_vec()))
        } else {
            None
        }
    }

    pub fn inc_render(&mut self, copy_oscillators: bool) {
        // Update the render index

        // Since self.renders has length 2, we can split it at index 1
        let (first, second) = self.renders.split_at_mut(1);

        let (current_render_option, next_render_option) = if self.render_idx == 0 {
            (&first[0], &mut second[0])
        } else {
            (&second[0], &mut first[0])
        };

        // Only copy oscillators if requested (for seamless transitions)
        if copy_oscillators {
            if let (Some(current_voices), Some(next_voices)) =
                (current_render_option.as_ref(), next_render_option.as_mut())
            {
                // Ensure that both renders have the same number of voices
                let min_length = std::cmp::min(current_voices.len(), next_voices.len());
                for i in 0..min_length {
                    let current_oscillator = &current_voices[i].oscillator;
                    let next_oscillator = &mut next_voices[i].oscillator;

                    next_oscillator.copy_state_from(current_oscillator);
                }
            }
        }

        // Reset samples processed for the new render
        self.samples_processed = 0;

        // Send visualization reset and audio ready events (non-blocking to prevent audio glitches)
        if self.events.render.has_subscribers() {
            self.events.render.emit(RenderEvent::Reset);
            self.events.render.emit(RenderEvent::AudioReady);
            // Pause until VisReady is received
            self.pause();
        }

        *self.current_render() = None;
        self.render_idx = (self.render_idx + 1) % 2;
    }

    pub fn current_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[self.render_idx]
    }

    pub fn next_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[(self.render_idx + 1) % 2]
    }

    pub fn current_render_ref(&self) -> &Option<Vec<RenderVoice>> {
        &self.renders[self.render_idx]
    }

    pub fn exists_current_render(&self) -> bool {
        self.renders[(self.render_idx) % 2].is_some()
    }

    pub fn exists_next_render(&self) -> bool {
        self.renders[(self.render_idx + 1) % 2].is_some()
    }

    pub fn push_render(&mut self, render: Vec<RenderVoice>, once: bool) {
        self.once = once;
        *self.next_render() = Some(render);
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
        self.exists_current_render()
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
        r.inc_render(true);
        assert_eq!(r.render_idx, 1);
        r.inc_render(true);
        assert_eq!(r.render_idx, 0);
    }

    fn render_voices_mock() -> Vec<RenderVoice> {
        vec![RenderVoice::init(&[RenderOp::init_silent_with_length(1.0)])]
    }

    #[test]
    fn test_push_render() {
        Settings::init_test();
        let mut r = RenderManager::init(None, None, false, None);
        assert_eq!(*r.current_render(), None);
        assert_eq!(*r.next_render(), None);
        r.push_render(render_voices_mock(), false);
        assert_eq!(*r.next_render(), Some(render_voices_mock()));
        assert_eq!(*r.current_render(), None);
        r.push_render(render_voices_mock(), false);
    }
}
