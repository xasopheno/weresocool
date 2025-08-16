 Big-picture layering you can move toward

  • Domain (pure): musical structures and time
    • Render-domain types: RenderOp, RenderVoiceCursor, StereoFrame, TimelineTime, EngineConfig.
    • Logic: batching/slicing, envelopes, panning, filters, ASR math, normalization to Op4D.
    • No globals, no IO, no threads.
  • Application services: orchestration
    • Engine/Renderer service that advances audio, schedules voices, mixes, handles volume ramps, looping policy.
    • Transport service (play/pause/seek/loop).
    • MidiMapper and VisualizationPublisher services producing events.
    • Ports/traits to talk to infrastructure.
  • Infrastructure (adapters)
    • AudioBackend (PortAudio/CPAL/WASM), MidiOut (UDP/CoreMIDI), VisualizerOut (crossbeam or web channel), FileWriter (wav/mp3/ogg).

  This split makes “other contexts” (WASM, offline render, server, DAW bridge) straightforward by swapping adapters.

  Obvious refactors from today’s code

  • Separate concerns currently bundled in RenderManager
    • Rendering, transport, volume ramping, MIDI emission, visualization, double-buffering, and configuration are all mixed.


   core/src/manager/render_manager.rs lines 79-93

    79 │ pub struct Visualization {
    80 │     normalizer: Normalizer,
    81 │     channel: VisualizationChannel,
    82 │ }
    83 │
    84 │ pub struct RenderManager {
    85 │     pub visualization: Visualization,
    86 │     pub renders: [Option<Vec<RenderVoice>>; 2],
    87 │     pub store: Option<Vec<Vec<RenderOp>>>,
    88 │     pub current_volume: f32,
    89 │     // ...
    90 │     midi_client: Option<midi_client::MidiClient>,
    91 │     midi_on: HashSet<(usize, usize, i64)>, // (voice, event, channel)
    92 │     midi_notes: HashMap<(usize, usize, i64), u8>, // note per key
    93 │ }

    • Extract:
      • Engine (advance/mix audio and state).
      • VolumeController (ramp smoothing).
      • RenderQueue (current/next render double-buffer logic).
      • MidiScheduler (derive MIDI note on/off from ops, handle channel mapping).
      • VisualizerPort (publish normalized Op4D).
  • Remove IO and UDP from the manager file


   core/src/manager/render_manager.rs lines 23-27

    23 │ mod midi_client {
    24 │     use serde::Serialize;
    25 │     use std::net::UdpSocket;
    26 │     // ...
    27 │ }

    • Replace with a trait:


   pub trait MidiOut { fn send(&self, msg: &MidiMsg); }

    • Provide UdpMidiOut in infra crate; pass it in via dependency injection.
  • Avoid global mutable configuration


   core/src/manager/render_manager.rs lines 196-196

   if let Some(s) = settings { Settings::init(s.sample_rate, s.buffer_size); } else { Settings::init_default(); }

    • Replace with an EngineConfig passed to services. Keep Settings as an immutable struct carried through or behind a SettingsProvider trait to ease
      testing.
  • Make the audio callback side-effect free and allocation free


   core/src/portaudio/real_time_render_manager.rs lines 18-18

   let batch = render_manager.lock().unwrap().read(...);

    • The callback should depend on a small, lock-free Engine interface that returns a pre-filled AudioBuffer and side-channel events. Use a lock-free
      ring buffer for inter-thread comms instead of Mutex where possible.
  • Iterator-based voice batching instead of recursion


   instrument/src/renderable/render_voice.rs lines 35-37

    35 │ pub fn get_batch(&mut self, samples_left_in_batch: usize, result: Option<Vec<RenderOp>>, loop_play: bool) -> Option<Vec<RenderOp>> {
    36 │     // recursive and allocates
    37 │ }

    • Provide a RenderCursor iterator that yields RenderSlice { op_ref, start_index, frames }. This improves testability and reduces allocations:


     1 │ pub struct RenderCursor<'a> { /* voice indices and refs */ }
     2 │ impl<'a> Iterator for RenderCursor<'a> {
     3 │   type Item = RenderSlice<'a>;
     4 │   fn next(&mut self) -> Option<Self::Item> { /* non-recursive */ }
     5 │ }

  • Decouple ramping from mixing


   core/src/manager/render_manager.rs lines 279-279

   fn ramp_to_current_volume(&mut self, buffer_size: usize) -> Vec<f32> { /* builds Vec */ }

    • Move into VolumeController with stateful advance(n_frames) -> GainEnvelope that can be applied in the backend or mixer. Avoid per-callback Vec
      allocation; reuse a scratch buffer or apply scalar when constant.
  • Make visualization a first-class port


   core/src/manager/render_manager.rs lines 525-525

   let mut opmap: OpMap<Op4D> = OpMap::with_capacity(ops.len()); /* normalize and send via channel */

    • Replace with VisualizerPort: fn publish(&self, &[Op4D]). Move normalization helper out of the manager.
  • Formalize Engine API around time and outputs
    • Current read returns audio and a ramp and also builds vis ops and MIDI:


   core/src/manager/render_manager.rs lines 318-318

   pub fn read(...) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)>

    • Replace with:


     1 │ pub struct EngineOutput<'a> {
     2 │   pub audio: AudioBuffer<'a>,            // interleaved frames or split
     3 │   pub events: SmallVec<[PerformanceEvent; 8]>, // midi/automation
     4 │   pub vis_ops: SmallVec<[Op4D; 64]>,
     5 │ }
     6 │ pub trait Engine {
     7 │   fn submit(&mut self, voices: Vec<RenderVoice>); // or RenderPlan
     8 │   fn advance(&mut self, frames: usize) -> EngineOutput;
     9 │   fn transport(&mut self) -> &mut dyn Transport; // play/pause/seek/loop
    10 │ }

    • Have infra backends consume EngineOutput and dispatch to adapters.
  • Unify PortAudio code paths; one “real-time” module
    • You have both real_time_render_manager.rs and an older commented real-time path. Keep one adapter that implements AudioBackend and calls
      engine.advance(buffer_size).
  • Type safety improvements in domain
    • Newtypes for pan, gain, frequency, seconds, frames to prevent unit confusion.
    • Replace raw Vec<u64> for WGSL and Vec<String> for colors with typed wrappers (WgslShaderId, ColorTag).
    • RenderOp could separate static parameters from per-slice state. Consider an immutable RenderEvent plus a lightweight RenderSlice.
  • Looping/queue policy as a strategy


   core/src/manager/render_manager.rs lines 616-616

   pub fn push_render(&mut self, render: Vec<RenderVoice>, once: bool)

    • Replace booleans with a PlaybackPolicy enum: Once, Loop, QueueNext, CrossfadeNext(f32).
  • Remove store experimental feature from core path


   core/src/manager/render_manager.rs lines 88-88

   pub store: Option<Vec<Vec<RenderOp>>>, /* TODO feature flag comments in loop */

    • Move behind a feature or into a separate RenderQueue service.


  Concrete API sketches

  • Domain: synthesizer boundary


     1 │ pub trait Synthesizer {
     2 │   fn prepare(&mut self, event: &RenderOp);
     3 │   fn render(&mut self, slice: &RenderSlice, offset: &FollowOffset, out: &mut AudioBufferMut);
     4 │ }

  • Application: engine and ports


     1 │ pub trait AudioBackend {
     2 │   fn start(self, engine: Arc<dyn Engine + Send + Sync>) -> Result<StreamHandle, Error>;
     3 │ }
     4 │
     5 │ pub trait MidiOut { fn send(&self, msg: &MidiMsg); }
     6 │ pub trait VisualizerOut { fn publish(&self, ops: &[Op4D]); }
     7 │
     8 │ pub trait Transport {
     9 │   fn play(&mut self);
    10 │   fn pause(&mut self);
    11 │   fn seek(&mut self, time_secs: f64);
    12 │   fn set_loop(&mut self, on: bool);
    13 │ }

  • Usage from CLI


     1 │ let engine = EngineBuilder::new(config)
     2 │   .with_midi(midi_out)
     3 │   .with_visualizer(vis_out)
     4 │   .build();
     5 │
     6 │ engine.submit(render_plan); // from parser
     7 │ audio_backend.start(engine)?;


  Incremental migration plan (safe steps)

  • Step 1: Extract traits and adapters without behavior changes
    • MidiOut + UdpMidiOut; inject into RenderManager.
    • VisualizerOut; move crossbeam sender behind it.
    • VolumeController; move ramp logic out.
  • Step 2: Replace Settings::global() with EngineConfig passed through; keep a thin global for compat behind a trait.
  • Step 3: Introduce RenderCursor iterator and use it inside RenderVoice::render_batch and RenderManager::read.
  • Step 4: Introduce Engine facade returning EngineOutput; adjust PortAudio adapter to consume it.
  • Step 5: Split the crate boundaries: move PortAudio into core-portaudio (infra), keep core as engine; migrate MIDI UDP to core-midi.
  • Step 6: Add an offline renderer implementing AudioBackend for file export and tests.


  Quick wins you can do first

  • Extract MIDI and visualization modules to adapters; inject via trait.
  • Stop allocating ramps every callback; reuse a buffer or apply scalar if unchanged.
  • Make RenderVoice::get_batch iterative and return a small fixed-capacity vector to avoid reallocations.
  • Replace Arc<Mutex<RenderManager>> in callback with a single-producer single-consumer ring buffer of AudioBuffer to eliminate locks.
  • Remove unwrap in the audio callback; propagate errors via a shared atomic flag and stop the stream.


  Where these issues are visible in code

  • Mixed concerns and UDP in manager:


   core/src/manager/render_manager.rs lines 472-477

   472 │ // Fire MIDI for current window ...
   473 │ if let Some(client) = &self.midi_client {
   474 │     // note mapping and delayed sends
   475 │     client.send(&midi_client::MidiMsg::NoteOnAt { ... });
   476 │     client.send(&midi_client::MidiMsg::NoteOffAt { ... });
   477 │ }

  • Global config and side effects in init:


   core/src/manager/render_manager.rs lines 196-196

   if let Some(s) = settings { Settings::init(s.sample_rate, s.buffer_size); } else { Settings::init_default(); }

  • Recursive batching and allocations:


   instrument/src/renderable/render_voice.rs lines 55-58

    55 │ if (current_op.samples - self.sample_index) > samples_left_in_batch {
    56 │     result.push(RenderOp { /* clone and push */ });
    57 │     self.sample_index += samples_left_in_batch;
    58 │ }
