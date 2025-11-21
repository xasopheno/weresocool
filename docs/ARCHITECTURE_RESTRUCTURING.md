# WereSoCool Architecture Restructuring Plan

**Version:** 1.0
**Date:** 2025-11-21
**Status:** Planning Phase

## Executive Summary

This document outlines a plan to restructure WereSoCool's architecture to:
- Introduce **middleware hooks** in the processing pipeline
- Add an **event system** for visualization, MIDI, and export
- Extract **reusable packages** for audio synthesis and real-time rendering
- Maintain **zero performance overhead** through compile-time abstractions

## Table of Contents

1. [Current Architecture](#current-architecture)
2. [Problems & Limitations](#problems--limitations)
3. [Proposed Architecture](#proposed-architecture)
4. [Detailed Component Design](#detailed-component-design)
5. [Migration Strategy](#migration-strategy)
6. [Performance Considerations](#performance-considerations)
7. [Benefits & Trade-offs](#benefits--trade-offs)

---

## Current Architecture

### Package Structure

WereSoCool currently consists of 13 crates organized as a workspace:

```
weresocool/
├── weresocool (CLI - main entry point)
├── weresocool_core (generation, rendering orchestration)
├── weresocool_ast (Abstract Syntax Tree, operations)
├── weresocool_parser (LALRPOP parser)
├── weresocool_instrument (audio synthesis, voice rendering)
├── weresocool_shared (settings, utilities)
├── scop (generic scope/namespace system)
├── opmap (operation mapping data structure)
├── weresocool_filter (biquad filters)
├── weresocool_error (error types)
├── weresocool_analyze (audio analysis)
├── weresocool_ring_buffer (ring buffer)
├── weresocool_lame (MP3 encoding)
└── weresocool_vorbis (OGG encoding)
```

### Data Flow Pipeline

```
.socool file
    ↓
Parser (LALRPOP) → ParsedComposition (Init + Defs)
    ↓
Normalize trait → NormalForm (Vec<Vec<PointOp>>)
    ↓
nf_to_vec_renderable() → Vec<Vec<RenderOp>>
    ↓
renderables_to_render_voices() → Vec<RenderVoice>
    ↓
RenderManager → real_time_render_manager()
    ↓
PortAudio callback → Audio output
```

**Parallel Visualization Pipeline:**
```
RenderOp → Op4D → crossbeam_channel → Viz renderer
```

### Key Design Patterns

**Traits:**
- `Normalize`, `Substitute`, `GetLengthRatio` - AST transformations
- `Renderable` - Audio rendering abstraction
- `Interpretable` - Unified file/string input

**Parallelism:**
- Rayon for parallel voice rendering
- Background thread for lookahead buffer filling
- Crossbeam channels for visualization events

**Coupling:**
- Core directly calls Instrument functions
- AST contains audio-specific types (OscType, filters)
- RenderManager tightly coupled to Voice implementation
- Defs threaded through entire pipeline as `&mut`

---

## Problems & Limitations

### 1. Tight Coupling

**Problem:** Core knows about Instrument internals, AST contains audio concerns.

**Impact:**
- Hard to swap synthesis engines
- Can't reuse components in other projects
- Testing requires full integration

### 2. No Middleware System

**Problem:** No way to plug custom transformations into the pipeline.

**Impact:**
- Custom operations require modifying core code
- Can't add preprocessing/postprocessing steps
- No extensibility for third-party plugins

### 3. Event Handling is Ad-hoc

**Problem:** Visualization and MIDI output are special-cased in RenderManager.

**Impact:**
- Hard to add new output formats
- No clear separation of concerns
- Difficult to test independently

### 4. Components Not Reusable

**Problem:** Synthesis engine and render manager are tightly bound to WereSoCool types.

**Impact:**
- Can't use the excellent Voice/Oscillator code in other projects
- Can't use the real-time rendering infrastructure elsewhere
- Missed opportunity for open-source contributions

### 5. Limited Extensibility

**Problem:** Adding new operations, generators, or effects requires deep knowledge of multiple crates.

**Impact:**
- High barrier to contribution
- No plugin ecosystem possible
- Monolithic design

---

## Proposed Architecture

### New Package Structure

```
weresocool/
├── CLI Layer
│   └── weresocool (CLI commands, user interface)
│
├── Language Layer
│   ├── weresocool_lang (NEW: AST + parser, no audio knowledge)
│   └── weresocool_error (error handling)
│
├── Pipeline Layer
│   ├── weresocool_core (NEW: middleware runner, event bus, orchestration)
│   └── weresocool_instrument (NEW: bridge from NormalForm → SynthOp)
│
├── Audio Layer (REUSABLE)
│   ├── weresocool_synth (NEW: generic synthesis engine) ⭐
│   └── weresocool_realtime (NEW: generic render manager) ⭐
│
├── Infrastructure Layer
│   ├── weresocool_shared (settings, utilities)
│   ├── scop (generic scope system)
│   ├── opmap (operation mapping)
│   └── weresocool_filter (biquad filters)
│
└── Export Layer
    ├── weresocool_analyze (analysis)
    ├── weresocool_lame (MP3)
    └── weresocool_vorbis (OGG)
```

### New Data Flow

```
.socool file
    ↓
Parser → ParsedComposition
    ↓
┌─────────────── Middleware Pipeline ───────────────┐
│  Hook: post-parse                                  │
│    ↓                                               │
│  Normalize → NormalForm                           │
│    ↓                                               │
│  Hook: post-normalize                             │
│    ↓                                               │
│  Bridge: NormalForm → Vec<SynthOp>                │
│    ↓                                               │
│  Hook: pre-render                                  │
└────────────────────────────────────────────────────┘
    ↓
Synth Engine → Vec<SynthVoice>
    ↓
Realtime Manager → Audio Backend
    ↓
    ├─→ PortAudio (output)
    ├─→ EventBus.emit(VisualizationEvent) → Viz handler
    ├─→ EventBus.emit(MidiEvent) → MIDI handler
    └─→ EventBus.emit(ExportEvent) → File handlers
```

---

## Detailed Component Design

### 1. `weresocool_synth` - Generic Synthesis Engine ⭐

**Purpose:** Reusable audio synthesis library, independent of WereSoCool language.

**Public API:**
```rust
// Core synthesis operation
pub trait SynthOp: Send + Sync {
    fn frequency(&self) -> f64;
    fn gain(&self) -> f64;
    fn pan(&self) -> f64;
    fn duration_samples(&self) -> usize;
    fn oscillator_type(&self) -> OscType;
    fn filters(&self) -> &[FilterOp];
    fn envelope(&self) -> Option<Envelope>;
}

// Voice that renders SynthOps
pub struct SynthVoice<Op: SynthOp> {
    oscillator: Oscillator,
    sample_rate: f64,
    // ... state
}

impl<Op: SynthOp> SynthVoice<Op> {
    pub fn render_batch(&mut self, ops: &[Op], buffer: &mut [f32]) -> usize;
}

// Re-export existing oscillator types
pub use self::{Oscillator, OscType, Sample, Voice};
```

**Migration:**
- Move `Voice`, `Oscillator`, `Sample` from `weresocool_instrument`
- Make them generic over `SynthOp` trait instead of concrete `RenderOp`
- Keep all performance optimizations (inline, tight loops)

**Zero-cost guarantee:** Monomorphization means `SynthVoice<RenderOp>` compiles to identical code as current `Voice`.

---

### 2. `weresocool_realtime` - Generic Render Manager ⭐

**Purpose:** Reusable real-time audio rendering infrastructure.

**Public API:**
```rust
// Backend abstraction
pub trait AudioBackend {
    type Sample;
    type Error;

    fn render_buffer(&mut self, buffer: &mut [Self::Sample]) -> Result<(), Self::Error>;
    fn sample_rate(&self) -> f64;
}

// Generic render manager
pub struct RealtimeManager<B: AudioBackend> {
    backend: B,
    buffer_manager: BufferManager,
    lookahead_buffers: usize,
    // ... state
}

impl<B: AudioBackend> RealtimeManager<B> {
    pub fn new(backend: B, settings: RealtimeSettings) -> Self;
    pub fn start(&mut self) -> Result<(), B::Error>;
    pub fn read(&mut self) -> StereoWaveform;
    pub fn set_volume(&mut self, vol: f32);
}

// PortAudio implementation
pub struct PortAudioBackend<R: Renderer> {
    renderer: R,
    // ... state
}
```

**Migration:**
- Move `RenderManager`, `BufferManager` from `weresocool_core`
- Make them generic over `AudioBackend` trait
- Keep parallel rendering (Rayon), lookahead buffers, volume ramping

**Zero-cost guarantee:** Generic backend monomorphizes, no virtual dispatch in audio callback.

---

### 3. `weresocool_core` - Middleware Runner & Event Bus

**Purpose:** Orchestrate pipeline, run middleware, dispatch events.

**Middleware System:**
```rust
// Compile-time middleware composition (zero overhead)
pub trait Middleware<T>: Send + Sync {
    type Output;

    fn process(&mut self, input: T) -> Result<Self::Output, Error>;
}

// Chainable pipeline
pub struct Pipeline<T> {
    stages: Vec<Box<dyn Middleware<T, Output = T>>>,
}

impl<T> Pipeline<T> {
    pub fn add<M: Middleware<T, Output = T> + 'static>(mut self, m: M) -> Self {
        self.stages.push(Box::new(m));
        self
    }

    pub fn run(&mut self, input: T) -> Result<T, Error> {
        self.stages.iter_mut().try_fold(input, |acc, stage| stage.process(acc))
    }
}

// Hook points
pub enum PipelineStage {
    PostParse,      // After parsing, before normalization
    PostNormalize,  // After normalization, before render conversion
    PreRender,      // After render conversion, before synthesis
}
```

**Event System:**
```rust
// Lock-free event bus
pub struct EventBus {
    viz_tx: Sender<VisualizationEvent>,
    midi_tx: Sender<MidiEvent>,
    export_tx: Sender<ExportEvent>,
}

pub trait EventHandler<E>: Send + Sync {
    fn handle(&mut self, event: E) -> Result<(), Error>;
}

impl EventBus {
    pub fn emit_viz(&self, event: VisualizationEvent) {
        let _ = self.viz_tx.try_send(event); // Non-blocking
    }

    pub fn subscribe_viz<H: EventHandler<VisualizationEvent>>(&mut self, handler: H);
}

// Concrete event types
pub struct VisualizationEvent {
    pub op4d: Op4D,
    pub timestamp: f64,
}

pub struct MidiEvent {
    pub note: u8,
    pub velocity: u8,
    pub timestamp: f64,
}
```

**Migration:**
- Extract pipeline coordination logic from `core/src/generation/`
- Move RenderManager to `weresocool_realtime`
- Create event bus, migrate viz/MIDI to event handlers
- Existing functionality becomes default middleware

---

### 4. `weresocool_lang` - Language Semantics

**Purpose:** AST and parser, no audio or rendering knowledge.

**Public API:**
```rust
// Pure language constructs
pub enum Op {
    Sequence(Vec<Term>),
    Overlay(Vec<Term>),
    // ... operations
}

pub trait Normalize {
    fn apply_to_normal_form(&self, nf: &mut NormalForm, defs: &mut Defs) -> Result<(), Error>;
}

// Output is language-level, not audio-level
pub struct NormalForm {
    pub voices: Vec<Voice>,
}

pub struct Voice {
    pub events: Vec<Event>,
}

pub struct Event {
    pub f_ratio: Rational,  // Not Hz
    pub g_ratio: Rational,  // Not amplitude
    pub l_ratio: Rational,  // Not samples
    // ... abstract operations
}
```

**Migration:**
- Move AST types from `weresocool_ast` to `weresocool_lang`
- Move parser from `weresocool_parser` to `weresocool_lang/parser`
- Remove audio-specific types (keep as annotations, not concrete values)
- `weresocool_instrument` becomes the bridge to audio

---

### 5. `weresocool_instrument` - Bridge Layer

**Purpose:** Convert language types to synthesis types.

**Public API:**
```rust
use weresocool_lang::{NormalForm, Basis};
use weresocool_synth::SynthOp;

// Bridge function
pub fn normal_form_to_synth_ops(
    nf: &NormalForm,
    basis: &Basis,
) -> Vec<Vec<impl SynthOp>> {
    // Convert abstract ratios to concrete Hz, samples, etc.
}

// Concrete SynthOp implementation
pub struct WscSynthOp {
    pub frequency: f64,  // Hz
    pub gain: f64,       // Amplitude
    pub pan: f64,
    pub duration_samples: usize,
    pub osc_type: OscType,
    pub filters: Vec<FilterOp>,
    pub envelope: Option<Envelope>,
}

impl SynthOp for WscSynthOp { /* ... */ }
```

**Migration:**
- Keep most of existing `weresocool_instrument`
- Implement `SynthOp` trait for `WscSynthOp` (renamed from `RenderOp`)
- Bridge between abstract language and concrete audio

---

## Migration Strategy

### Phase 1: Extract Synthesis Engine (Low Risk)

**Goal:** Create `weresocool_synth` without breaking existing code.

**Steps:**
1. Create new `weresocool_synth` crate
2. Define `SynthOp` trait
3. Copy `Voice`, `Oscillator`, `Sample` from `instrument/src/`
4. Make them generic over `SynthOp` instead of concrete `RenderOp`
5. Update `weresocool_instrument` to use `weresocool_synth`
6. Implement `SynthOp` for existing `RenderOp`

**Verification:**
```bash
# Benchmark voice rendering before
cargo bench --bench voice_rendering > before.txt

# After migration
cargo bench --bench voice_rendering > after.txt

# Should be identical (within noise)
diff before.txt after.txt
```

**Rollback:** Keep old code in place until benchmarks pass.

---

### Phase 2: Extract Render Manager (Medium Risk)

**Goal:** Create `weresocool_realtime` with generic backend.

**Steps:**
1. Create new `weresocool_realtime` crate
2. Define `AudioBackend` trait
3. Move `RenderManager`, `BufferManager` from `core/src/manager/`
4. Make them generic over `AudioBackend`
5. Create `PortAudioBackend` implementation
6. Update `weresocool_core` to use `weresocool_realtime`

**Verification:**
```bash
# Test real-time rendering
cargo run -- play mocks/victory.socool

# Should sound identical, no dropouts
# Check CPU usage is same: top -pid $(pgrep weresocool)
```

**Rollback:** Keep `#[cfg(feature = "old_manager")]` gate until verified.

---

### Phase 3: Add Middleware System (Medium Risk)

**Goal:** Add middleware hooks without changing existing behavior.

**Steps:**
1. Add `Pipeline` and `Middleware` traits to `core/src/middleware/`
2. Identify insertion points: post-parse, post-normalize, pre-render
3. Create default middleware that replicate current behavior
4. Insert `pipeline.run()` calls in existing flow
5. Verify output is identical

**Verification:**
```bash
# Export to WAV and compare byte-for-byte
cargo run -- print mocks/victory.socool -f test_before.wav
# After migration
cargo run -- print mocks/victory.socool -f test_after.wav
diff test_before.wav test_after.wav  # Should be identical
```

**Rollback:** Middleware system is additive, can be disabled with `--no-middleware` flag during testing.

---

### Phase 4: Add Event System (Low Risk)

**Goal:** Replace direct viz/MIDI calls with event emissions.

**Steps:**
1. Create `EventBus` with typed channels
2. Define `VisualizationEvent`, `MidiEvent`, `ExportEvent`
3. Create event handlers for existing viz/MIDI functionality
4. Replace direct calls with `event_bus.emit()`
5. Subscribe handlers on startup

**Verification:**
```bash
# Play with visualization
cargo run -- play mocks/victory.socool

# Visualization should work identically
# MIDI output should work identically
```

**Rollback:** Keep old code path behind `#[cfg(feature = "direct_calls")]`.

---

### Phase 5: Refactor Language Layer (High Risk)

**Goal:** Separate language semantics from audio concerns.

**Steps:**
1. Create `weresocool_lang` crate
2. Move AST types, keep them abstract (ratios, not Hz/samples)
3. Move parser
4. Update `weresocool_instrument` bridge to do concrete conversions
5. Update all imports across codebase

**Verification:**
```bash
# Full test suite must pass
cargo test --all

# All examples must still work
./scripts/test_all_examples.sh
```

**Rollback:** Large refactor, use git branch and thorough testing before merge.

---

## Performance Considerations

### Zero-Cost Abstractions

**Technique:** Trait monomorphization at compile time.

**Example:**
```rust
// This trait...
pub trait SynthOp {
    fn frequency(&self) -> f64;
}

// ...with this impl...
impl SynthOp for WscSynthOp {
    #[inline(always)]
    fn frequency(&self) -> f64 { self.frequency }
}

// ...and this generic...
pub fn render<Op: SynthOp>(op: &Op) -> f32 {
    let freq = op.frequency();
    // ...
}

// ...compiles to same assembly as direct access
// render(&wsc_op) ≈ wsc_op.frequency
```

**Verification:** Use `cargo-asm` or `cargo-llvm-lines` to inspect codegen.

---

### Parallel Rendering Preserved

**Current:** Rayon parallel voice rendering, background buffer filling.

**After migration:**
```rust
// In weresocool_synth
impl<Op: SynthOp> SynthVoice<Op> {
    pub fn render_batch(&mut self, ops: &[Op], buffer: &mut [f32]) -> usize {
        // Same tight loop as before
    }
}

// In weresocool_realtime
impl<B: AudioBackend> RealtimeManager<B> {
    fn fill_buffers(&mut self) {
        // Still uses Rayon for parallel voices
        use rayon::prelude::*;
        voices.par_iter_mut()
            .map(|voice| voice.render_batch(...))
            .collect()
    }
}
```

**Guarantee:** Rayon usage unchanged, same parallelism strategy.

---

### No Audio Thread Allocations

**Current:** Audio callback reads pre-rendered buffers from queue, no allocation.

**After migration:**
```rust
// In weresocool_realtime
impl<B: AudioBackend> RealtimeManager<B> {
    // Audio thread calls this
    pub fn read(&mut self) -> StereoWaveform {
        // Option 1: Pop from pre-filled queue (no allocation)
        if let Some(buffer) = self.buffer_queue.pop() {
            return buffer;
        }

        // Option 2: Render on-demand using pre-allocated buffer
        self.scratch_buffer.clear(); // Reuse allocation
        self.backend.render_buffer(&mut self.scratch_buffer);
        // ...
    }
}
```

**Guarantee:**
- Lookahead buffers pre-allocated on startup
- Event channels pre-allocated (crossbeam bounded channel)
- Scratch buffers reused, never freed in audio thread

---

### Middleware Overhead

**Goal:** Zero overhead for empty middleware, minimal for active middleware.

**Design:**
```rust
// Empty pipeline compiles away
let pipeline = Pipeline::new();
let result = pipeline.run(input);  // Optimized to: let result = input;

// Active middleware inlines
let pipeline = Pipeline::new()
    .add(NormalizeMiddleware::new())
    .add(ValidateMiddleware::new());
let result = pipeline.run(input);

// Compiles to:
// let result = input;
// let result = NormalizeMiddleware::process(result);
// let result = ValidateMiddleware::process(result);
// No vtable, no indirection
```

**Caveat:** Dynamic middleware (`Vec<Box<dyn Middleware>>`) has vtable cost. Use generics where possible.

---

### Event System Overhead

**Design:**
```rust
// Non-blocking send
event_bus.emit_viz(event);  // try_send, never blocks audio thread

// Pre-allocated bounded channel
let (tx, rx) = crossbeam::channel::bounded(1024);  // Fixed capacity

// Handler runs in separate thread
thread::spawn(move || {
    for event in rx {
        handler.handle(event);  // Async from audio thread
    }
});
```

**Guarantee:**
- Audio thread never waits for handlers
- If channel full, event dropped (acceptable for viz/MIDI)
- No allocation in emit path

---

### Benchmark Suite

**Add benchmarks for:**
1. Voice rendering (per-voice, per-sample cost)
2. Full pipeline (parse → render)
3. Real-time rendering (buffer fill latency)
4. Memory usage (peak allocation, steady-state)

**CI Integration:**
```bash
# Run benchmarks on every PR
cargo bench --all

# Compare against main branch
cargo benchcmp main current
```

---

## Benefits & Trade-offs

### Benefits

**1. Modularity**
- ✅ Synthesis engine reusable in other Rust audio projects
- ✅ Real-time manager reusable for any streaming audio app
- ✅ Clear boundaries between language, pipeline, and audio

**2. Extensibility**
- ✅ Middleware allows custom transformations without forking
- ✅ Event system allows custom handlers (e.g., OSC output, network streaming)
- ✅ Plugin ecosystem possible in future

**3. Testability**
- ✅ Each layer testable independently
- ✅ Mock backends for testing without audio hardware
- ✅ Easier to write unit tests vs. integration tests

**4. Documentation**
- ✅ Reusable crates incentivize better docs
- ✅ Clear API boundaries easier to document
- ✅ Examples show standalone usage

**5. Community**
- ✅ Standalone crates attract contributors outside WereSoCool
- ✅ Potential for crates.io publication
- ✅ Cross-pollination with other audio projects

---

### Trade-offs

**1. More Complexity**
- ⚠️ More crates to navigate
- ⚠️ Generics can be harder to read
- 🔧 *Mitigation:* Comprehensive docs, clear naming, examples

**2. Longer Compile Times (Potentially)**
- ⚠️ More generic code = more monomorphization
- ⚠️ More crates = more parallel compilation units
- 🔧 *Mitigation:* Use `sccache`, optimize dependencies, measure impact

**3. Migration Risk**
- ⚠️ Large refactor could introduce bugs
- ⚠️ Performance regressions possible if not careful
- 🔧 *Mitigation:* Phased migration, extensive benchmarks, rollback plans

**4. Maintenance Burden**
- ⚠️ More public APIs to maintain
- ⚠️ Backward compatibility constraints if crates published
- 🔧 *Mitigation:* Use semantic versioning, keep internal crates private initially

**5. Over-engineering Risk**
- ⚠️ Middleware might be overkill if not heavily used
- ⚠️ Generics add cognitive overhead
- 🔧 *Mitigation:* YAGNI principle - only add abstractions when needed, keep defaults simple

---

## Success Criteria

### Must Have

- ✅ All existing functionality works identically
- ✅ Performance benchmarks within 5% of baseline
- ✅ No regressions in audio quality or latency
- ✅ All tests pass
- ✅ CLI interface unchanged

### Should Have

- ✅ `weresocool_synth` usable in standalone audio project (example provided)
- ✅ `weresocool_realtime` usable in standalone streaming app (example provided)
- ✅ At least one middleware example (e.g., debug logging middleware)
- ✅ At least one event handler example (e.g., OSC output handler)

### Nice to Have

- ⭐ Published to crates.io
- ⭐ External contributor uses standalone crate
- ⭐ Performance improvement from optimizations discovered during refactor
- ⭐ Plugin system documentation

---

## Open Questions

1. **Should middleware support async?**
   - Current: Sync only
   - Trade-off: Async adds complexity but enables I/O middleware (network, file)

2. **Should we use trait objects or generics for event handlers?**
   - Current plan: Generics for zero-cost
   - Alternative: Trait objects for runtime registration

3. **Should synthesis engine support GPU acceleration?**
   - Not in scope for initial refactor
   - Architecture should allow future addition

4. **Should we extract visualization renderer to separate crate?**
   - Currently tightly coupled to OpenGL
   - Potentially useful standalone, but lower priority

5. **Should language layer support IR (intermediate representation)?**
   - Could enable optimizations passes
   - Significant scope increase

---

## Next Steps

1. **Review this document** with maintainers/collaborators
2. **Create benchmarks** for baseline performance
3. **Phase 1 implementation:** Extract synthesis engine
4. **Validate Phase 1:** Run benchmarks, verify audio output
5. **Iterate** through remaining phases

---

## References

- **Current codebase:** `/Users/danny/code/weresocool`
- **Branch for refactor:** `dm/architecture-refactor` (to be created)
- **Benchmark data:** `benches/` (to be created)
- **Example projects:** `examples/standalone/` (to be created)

---

**Document maintained by:** Architecture team
**Last updated:** 2025-11-21
**Status:** Ready for review
