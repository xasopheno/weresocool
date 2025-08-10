MIDI output design (initial)

- Goal: Allow piping any op sequence to one or more MIDI 1.0 channels using language pipe op `Midi(10)` or `Midi[1,10]`.
- Parser: Added `Midi` terminals in `parser/src/socool.lalrpop` under BaseOperation.
- AST: Added `Op::Midi { channels: Vec<i64> }` and `MidiTarget` (placeholder for future structure), kept channels as signed ints for parser simplicity.
- AST PointOp: Added `midi: Vec<i64>` field. Normalization (`ast/src/operations/normalize.rs`) appends channels to each `PointOp` when `Midi{...}` is applied.
- RenderOp: Added `midi: Vec<i64>` and threaded it through `pointop_to_renderop`, batching, and cloning.
- Render loop: In `core::manager::render_manager`, split render batches into two paths:
  - audio path (no midi channels) continues to audio synthesis
  - midi path collects `RenderOp`s with non-empty `midi` and emits UDP JSON messages to local MIDI server
- Transport: Simple UDP client to `127.0.0.1:6479`, JSON messages `{ type: "NoteOn"|"NoteOff", ch, note, vel }`.
  - Frequency to note uses nearest-note rounding; no bends now.
  - Channels are interpreted as 1..16 in the language; converted to 0-based for MIDI.
- State: Keeps a set of active (voice,event,channel) to send NoteOffs when events drop out of the current window.

Next
- Implement a lightweight JSON UDP listener in `weresocool_midi` that creates a MIDI 1.0 virtual output and forwards NoteOn/NoteOff.
- Consider NoteOff timing using op lengths rather than window diffs. For now, events retrigger window-to-window.
- Add test/fixture demonstrating `Seq [...] | Midi(10)` producing MIDI and silencing audio for those ops (current behavior: MIDI ops are not synthesized to audio).
- Add config for MIDI endpoint (host/port, A4) via `Settings` or env.
