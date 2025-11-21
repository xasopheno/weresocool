use crate::generation::Normalizer;
use crate::manager::resizeable_2d_vec::Resizeable2DVec;
use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    generation::sum_all_waveforms,
    generation::Op4D,
    interpretable::{InputType, Interpretable},
};
use log::info;
use opmap::OpMap;
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::{path::PathBuf, sync::mpsc::SendError};
use weresocool_ast::{Defs};
use weresocool_ast::follow::evaluate::EvaluateAction;
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, render_voice::renderables_to_render_voices, Offset, RenderOp, render_voice::RenderVoice, Renderable,
};
use weresocool_instrument::StereoWaveform;
use weresocool_shared::Settings;

mod midi_client {
    use serde::Serialize;
    use std::net::UdpSocket;

    #[derive(Debug)]
    pub struct MidiClient {
        socket: UdpSocket,
        addr: String,
    }

    impl MidiClient {
        pub fn new(addr: &str) -> std::io::Result<Self> {
            let socket = UdpSocket::bind("127.0.0.1:0")?; // ephemeral port
            socket.set_nonblocking(true)?;
            Ok(Self { socket, addr: addr.to_string() })
        }

        pub fn send(&self, msg: &impl Serialize) {
            if let Ok(buf) = serde_json::to_vec(msg) {
                if let Err(e) = self.socket.send_to(&buf, &self.addr) {
                    eprintln!("weresocool: failed to send MIDI UDP: {}", e);
                }
            }
        }
    }

    #[derive(Serialize)]
    #[serde(tag = "type")]
    pub enum MidiMsg {
        NoteOn { ch: u8, note: u8, vel: u8 },
        NoteOff { ch: u8, note: u8, vel: u8 },
        Pan { ch: u8, value: u8 },
        NoteOnAt { ch: u8, note: u8, vel: u8, delay_ms: u64 },
        NoteOffAt { ch: u8, note: u8, vel: u8, delay_ms: u64 },
        PanAt { ch: u8, value: u8, delay_ms: u64 },
        Expr { ch: u8, value: u8 },
        ExprAt { ch: u8, value: u8, delay_ms: u64 },
    }

    pub fn freq_to_midi_note(freq_hz: f64, a4: f64) -> u8 {
        if freq_hz <= 0.0 { return 0; }
        let midi = 69.0 + 12.0 * (freq_hz / a4).log2();
        midi.round().clamp(0.0, 127.0) as u8
    }
}

pub type KillChannel = Option<Sender<bool>>;

#[derive(Debug, Clone)]
pub enum VisEvent {
    Ops(opmap::OpMap<Op4D>),
    Reset,
    AudioReady,
    VisReady,
}
pub type VisualizationChannel = Option<crossbeam_channel::Sender<VisEvent>>;

#[derive(Debug, Clone)]
pub struct PrerenderedBuffer {
    pub waveform: StereoWaveform,
    pub ramp: Vec<f32>,
    pub ops: Vec<Vec<RenderOp>>,
}

#[derive(Debug)]
pub struct Visualization {
    normalizer: Normalizer,
    channel: VisualizationChannel,
    reverse_channel: Option<crossbeam_channel::Receiver<VisEvent>>,
}

#[derive(Debug)]
pub struct RenderManager {
    pub visualization: Visualization,
    pub renders: [Option<Vec<RenderVoice>>; 2],
    pub store: Option<Vec<Vec<RenderOp>>>,
    pub current_volume: f32,
    pub past_volume: f32,
    render_idx: usize,
    _read_idx: usize,
    kill_channel: KillChannel,
    once: bool,
    paused: bool,
    total_samples_per_loop: usize,
    samples_processed: usize,
    // MIDI
    midi_client: Option<midi_client::MidiClient>,
    midi_on: HashSet<(usize, usize, i64)>, // (voice, event, channel)
    midi_notes: HashMap<(usize, usize, i64), u8>, // note per key
    // Double buffering
    buffer_sender: Option<crossbeam_channel::Sender<PrerenderedBuffer>>,
    buffer_receiver: Option<crossbeam_channel::Receiver<PrerenderedBuffer>>,
    render_thread: Option<std::thread::JoinHandle<()>>,
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
        visualization_channel: VisualizationChannel,
        reverse_channel: Option<crossbeam_channel::Receiver<VisEvent>>,
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
        let (buffer_sender, buffer_receiver) = if lookahead_buffers > 0 {
            let (s, r) = crossbeam_channel::bounded(lookahead_buffers);
            (Some(s), Some(r))
        } else {
            (None, None)
        };

        Self {
            visualization: Visualization {
                channel: visualization_channel,
                reverse_channel,
                normalizer: Normalizer::default(),
            },
            renders: [None, None],
            store: None,
            past_volume: 0.8,
            current_volume: 0.8,
            render_idx: 0,
            _read_idx: 0,
            kill_channel,
            once,
            paused: false,
            total_samples_per_loop: 0,
            samples_processed: 0,
            midi_client: midi_client::MidiClient::new("127.0.0.1:6479").ok(),
            midi_on: HashSet::new(),
            midi_notes: HashMap::new(),
            buffer_sender,
            buffer_receiver,
            render_thread: None,
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
        let (buffer_sender, buffer_receiver) = if lookahead_buffers > 0 {
            let (s, r) = crossbeam_channel::bounded(lookahead_buffers);
            (Some(s), Some(r))
        } else {
            (None, None)
        };

        Self {
            visualization: Visualization {
                channel: None,
                normalizer: Normalizer::default(),
                reverse_channel: None,
            },
            renders: [None, None],
            store: None,
            past_volume: 0.8,
            current_volume: 0.8,
            render_idx: 0,
            _read_idx: 0,
            kill_channel: None,
            once: false,
            paused: false,
            total_samples_per_loop: 0,
            samples_processed: 0,
            midi_client: None,
            midi_on: HashSet::new(),
            midi_notes: HashMap::new(),
            buffer_sender,
            buffer_receiver,
            render_thread: None,
        }
    }

    pub fn kill(&self) -> Result<(), SendError<bool>> {
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
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Check for VisReady event and unpause if received (non-blocking)
    pub fn check_vis_ready(&mut self) -> bool {
        if let Some(rx) = &self.visualization.reverse_channel {
            if let Ok(event) = rx.try_recv() {
                if matches!(event, VisEvent::VisReady) {
                    self.play();
                    return true;
                }
            }
        }
        false
    }

    pub fn update_volume(&mut self, volume: f32) {
        self.current_volume = f32::powf(volume, 2.0)
    }

    fn ramp_to_current_volume(&mut self, buffer_size: usize) -> Vec<f32> {
        let mut offset: Vec<f32> = Vec::with_capacity(buffer_size * 2);
        let distance = self.current_volume - self.past_volume;

        // Use crossfade_period for smoother volume transitions
        let crossfade_samples = weresocool_shared::Settings::global().crossfade_period;

        // If we're far from target volume, use longer crossfade
        let ramp_length = if distance.abs() > 0.3 {
            crossfade_samples.max(buffer_size * 2)
        } else {
            buffer_size * 2
        };

        let denom = ramp_length as f32;
        for i in 0..(buffer_size * 2) {
            if i < ramp_length {
                offset.push(self.past_volume + (distance * i as f32 / denom));
            } else {
                offset.push(self.current_volume);
            }
        }

        // Only update past_volume if we've reached the target
        if buffer_size * 2 >= ramp_length {
            self.past_volume = self.current_volume;
        } else {
            self.past_volume += distance * (buffer_size * 2) as f32 / denom;
        }

        offset
    }

    pub fn push_ops_to_store(&mut self, to_store: Vec<Vec<RenderOp>>) {
        if let Some(store) = &mut self.store {
            if store.len() < to_store.len() {
                // Extend the store to match the size of `to_store`
                store.extend((store.len()..to_store.len()).map(|_| Vec::new()));
            }

            store.iter_mut().zip(to_store).for_each(|(voice, ops)| {
                voice.extend(ops.into_iter().map(|mut op| {
                    op.follows = vec![];
                    op
                }));
            });
        } else {
            self.store = Some(to_store);
        }
    }

    pub fn push_store_to_current_render(&mut self) {
        // Take the store out temporarily
        if let Some(store) = self.store.take() {
            if let Some(current_render) = self.current_render().as_mut() {
                current_render.extend(store.into_iter().map(|ops| RenderVoice::init(&ops.clone())));
            }
            self.store = None
        }
    }

    /// Pop a pre-rendered buffer from the queue (non-blocking, for audio thread)
    pub fn pop_buffer(&self) -> Option<PrerenderedBuffer> {
        if let Some(receiver) = &self.buffer_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    /// Drain all buffers from the queue (use when switching renders to clear stale audio)
    pub fn drain_buffer_queue(&self) {
        if let Some(receiver) = &self.buffer_receiver {
            while receiver.try_recv().is_ok() {
                // Discard all buffers
            }
        }
    }

    /// Start background rendering thread that pre-renders buffers
    /// Returns the JoinHandle for the rendering thread
    pub fn start_background_rendering(render_manager: Arc<Mutex<RenderManager>>) -> std::thread::JoinHandle<()> {
        let rm_clone = Arc::clone(&render_manager);

        // Get the sender once at the start, outside the loop
        let sender = {
            let rm = rm_clone.lock().unwrap();
            rm.buffer_sender.clone()
        };

        std::thread::Builder::new()
            .name("weresocool-render".to_string())
            .spawn(move || {
                let Some(sender) = sender else {
                    return; // No sender, exit thread
                };

                loop {
                    // Try to render the next buffer
                    let (should_continue, buffer_result) = {
                        let mut rm = rm_clone.lock().unwrap();

                        // Check if we should stop
                        if rm.paused {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            (true, None)
                        } else {
                            // Render a buffer
                            let buffer_size = Settings::global().buffer_size;
                            let result = rm.read(
                                buffer_size,
                                Offset {
                                    freq: 1.0,
                                    gain: 1.0,
                                },
                            );

                            let should_continue = result.is_some() || rm.exists_current_render();
                            (should_continue, result)
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

        let vtx = self.visualization.channel.clone();
        let normalizer = self.visualization.normalizer;

        // Track absolute sample position at the start of this read
        let mut read_start_samples = self.samples_processed;
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
                            // TODO: Should return if it looped and reset store
                            // TODO: or should it just push the store to the current render?
                            // TODO: This is getting super complicated...what should I do?
                            // TODO: Maybe factor this out?
                            // TODO: How do I save the state so I can print?
                            // TODO: The store stuff should be behind a feature flag

                            match voice.get_batch(
                                remaining_buffer_size,
                                None,
                                !next_exists && Settings::global().loop_play,
                            ) {
                                Some(mut batch) => {
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

                                    if let Some(_vtx) = &vtx {
                                        // let b = batch
                                        // .clone()
                                        // .into_iter()
                                        // .map(|mut op| {
                                        // if op.follow {
                                        // op.f = op.f * offset.freq;
                                        // op.g = (
                                        // op.g.0 * offset.gain,
                                        // op.g.1 * offset.gain,
                                        // );
                                        // }
                                        // op
                                        // })
                                        // .collect();
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
                    // self.push_store_to_current_render();
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

        // Fire MIDI for current window using op.t timing (NoteOn only at op start; NoteOff at op end)
        if let Some(client) = &self.midi_client {
            let a4 = 440.0f64;
            let sr = Settings::global().sample_rate as f64;
            for op in &midi_ops {
                if op.midi.is_empty() { continue; }

                // Voice→channel mapping (overlay):
                let ch1: u8 = if op.midi.len() == 1 {
                    let base = op.midi[0].max(1).min(16);
                    (((base - 1) as usize + op.voice) % 16) as u8 + 1
                } else {
                    op.midi[op.voice % op.midi.len()]
                };
                let ch = (ch1.saturating_sub(1)).min(15);

                let note = midi_client::freq_to_midi_note(op.f, a4);
                let pan_val = (((op.p + 1.0) / 2.0) * 127.0).round().clamp(0.0, 127.0) as u8;

                let is_start = op.index == 0;
                let is_end = op.index + op.samples >= op.total_samples;

                // Compute delays relative to this read start, based on op.t (seconds)
                let start_samples = (op.t * sr).round() as usize;
                let end_samples = start_samples.saturating_add(op.total_samples);
                let delay_on_ms = if start_samples > read_start_samples {
                    ((start_samples - read_start_samples) as f64 / sr * 1000.0).round() as u64
                } else { 0 };
                let delay_off_ms = if end_samples > read_start_samples {
                    ((end_samples - read_start_samples) as f64 / sr * 1000.0).round() as u64
                } else { 0 };

                if is_start {
                    // Use velocity to control per-hit loudness (works best for drums)
                    // Map op.gain_scalar in [0.0..2.0] to [1..127], with Gm 1.0 -> 100
                    let mut vel_f = (op.gain_scalar * 100.0).round();
                    if vel_f < 1.0 { vel_f = 1.0; }
                    if vel_f > 127.0 { vel_f = 127.0; }
                    let vel = vel_f as u8;
                    client.send(&midi_client::MidiMsg::PanAt { ch, value: pan_val, delay_ms: delay_on_ms });
                    client.send(&midi_client::MidiMsg::NoteOnAt { ch, note, vel: vel, delay_ms: delay_on_ms });
                }
                if is_end {
                    client.send(&midi_client::MidiMsg::NoteOffAt { ch, note, vel: 64, delay_ms: delay_off_ms });
                }
            }
        }

        // If we rendered anything, pad to the buffer size and return
        if combined_sw.l_buffer.len() > 0 {
            combined_sw.pad(buffer_size);

            // Visualization
            if let Some(tx) = vtx {
                let ops = total_ops.to_vec_flat();
                let mut opmap: OpMap<Op4D> = OpMap::with_capacity(ops.len());
                ops.iter().for_each(|v| {
                    let name = v.colors.last().map_or("nameless", |n| n);

                    let op = render_op_to_normalized_op4d(v, &normalizer);
                    if let Some(o) = op {
                        opmap.insert(name, o);
                    };
                });

                if tx.send(VisEvent::Ops(opmap)).is_err() {
                    info!("Visualization channel closed");
                    std::process::exit(0);
                }
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
        if let Some(vtx) = self.visualization.channel.clone() {
            let _ = vtx.try_send(VisEvent::Reset);
            let _ = vtx.try_send(VisEvent::AudioReady);
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
        let mut rm = RenderManager::init(None, None, None, false, None);
        rm.update_volume(0.9);
        assert!(cmp_f32(rm.current_volume, f32::powf(0.9, 2.0)));
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
        let mut r = RenderManager::init(None, None, None, false, None);
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
        let mut r = RenderManager::init(None, None, None, false, None);
        assert_eq!(*r.current_render(), None);
        assert_eq!(*r.next_render(), None);
        r.push_render(render_voices_mock(), false);
        assert_eq!(*r.next_render(), Some(render_voices_mock()));
        assert_eq!(*r.current_render(), None);
        r.push_render(render_voices_mock(), false);
    }
}
