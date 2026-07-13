//! AST for the draw DSL.
//!
//! A draw is a `|`-piped chain that starts with a generator (`Point`) and runs
//! ops left-to-right, each transforming the emit-set the previous op produced.
//! `Each(sub_pipeline)` is the recursion combinator — for each emit in the
//! current set, run `sub_pipeline` rebound to that emit's position, union the
//! results. Numeric expressions can reference per-note properties
//! (`note.x`, `note.y`, `note.z`, `note.l`, `note.t`, `note.event`) and the
//! live `clock`.
//!
//! See `docs/draw-dsl-design.md`.

// The expression sublanguage is shared across the visual DSLs — see
// `crate::dsl_expr` (DSL_STYLE Law 4). `DrawExpr` is an alias so draw keeps its
// spelling while the AST stays single-source. draw's grammar produces the
// full set MINUS `UserParam` (warp-only): `Lit`/`Clock`/`Note`/`Stroke`/`Sin`/
// `Bin`. `Note(field)` is `note.x` etc.; `Stroke` is `0.0`→`1.0` along the
// current expansion (only meaningful per-emit, inside `Tint`).
pub use crate::dsl_expr::{BinOp, Expr as DrawExpr, MathFn, NoteField};

/// Cardinal axes used by `Mirror`, rotations, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis { X, Y, Z }

/// 3D positions in the draw DSL. All resolve from the source note (no
/// `Next`/`Prev` — see the "deliberately deferred" section of the spec).
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// The current emit's position (rebound by `Each`).
    Here,
    /// Single-axis offsets from `Here`, same idiom as wgsl-DSL `Xa`/`Ya`/`Za`.
    Xa(DrawExpr),
    Ya(DrawExpr),
    Za(DrawExpr),
    /// Explicit vec3 displacement from `Here`.
    Direction(DrawExpr, DrawExpr, DrawExpr),
    /// Vector arithmetic: `Anchor + Anchor`, `Anchor - Anchor`.
    Add(Box<Anchor>, Box<Anchor>),
    Sub(Box<Anchor>, Box<Anchor>),
}

/// Per-copy delta used inside `Spawn(n, by: Increment)`. The verb vocabulary
/// IS the wgsl-DSL set, applied accumulatively per copy.
#[derive(Debug, Clone, PartialEq)]
pub enum Increment {
    Xa(DrawExpr),
    Ya(DrawExpr),
    Za(DrawExpr),
    Direction(DrawExpr, DrawExpr, DrawExpr),
    /// Rotation around an axis, in TURNS — `1.0` = 360° (`Rz(1/8)` = one eighth turn).
    Rx(DrawExpr),
    Ry(DrawExpr),
    Rz(DrawExpr),
    Sm(DrawExpr),
    Sa(DrawExpr),
    /// Apply multiple increments in parallel each step.
    Tuple(Vec<Increment>),
}

/// Field expressions used by `Modulate(field, amount)`. Same expression
/// sublanguage as warp's `Source` variants, with note-property accessors added.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Sin { freq: DrawExpr },
    /// Independent random sample per emit — scatters an expansion's emits into
    /// a fuzzy painterly cloud. NOT spatially coherent (use `Flow` for that).
    Noise { scale: DrawExpr, seed: Option<DrawExpr> },
    /// Spatially-coherent 3D value noise sampled at the emit's position ×
    /// `scale`. Nearby emits get nearby displacement → directional, flow-like
    /// turbulence (the CurlFlow look) that keeps a stroke's points related
    /// rather than scattering them. `seed` is per-note-stable.
    Flow { scale: DrawExpr, seed: Option<DrawExpr> },
    Const(DrawExpr),
    Note(NoteField),
}

/// One op in a draw pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawOp {
    // === Form / shape ===
    /// Replace the emit set with an explicit polyline of points — the way to
    /// describe a deliberate FORM (a fish, a letter, a triangle) rather than a
    /// generative texture. One note stamps the whole shape. Coords are
    /// expressions in the note's local frame (so they can reference note
    /// fields); pair with `Pin` to place the form in absolute world space.
    /// Written `Path [ (x, y), (x, y, z), … ]` at the start of a pipeline.
    Path(Vec<(DrawExpr, DrawExpr, DrawExpr)>),
    /// Resample the current ordered emit set as a Catmull-Rom spline through
    /// its points to `n` samples — smooth organic forms from a few controls.
    Smooth(DrawExpr),
    /// Reduce the ordered set to the SINGLE point at fraction `t` (0→1) along
    /// it (interpolated; `t` wraps mod 1). This is how a melody PASSES THROUGH
    /// a path instead of stamping the whole form: drive `t` by a note field so
    /// each note lands at its position on the track —
    /// `Path […] | Smooth(n) | Pin | Pick(note.event / k)` walks the curve as
    /// the melody plays (warp Decay draws the trail). Map pitch with
    /// `Pick((note.y + 1)/2)` to place by height instead of sequence.
    Pick(DrawExpr),
    /// Mark every emit ABSOLUTE: its offset is a world position, not relative
    /// to the note's (pan, pitch). Lets a form hold still in space while the
    /// notes of the voice it's routed to redraw and animate it.
    Pin,

    // === Set translations (move every emit) ===
    Xa(DrawExpr),
    Ya(DrawExpr),
    Za(DrawExpr),
    Direction(DrawExpr, DrawExpr, DrawExpr),
    // === Set rotations (rotate every emit around origin) ===
    Rx(DrawExpr),
    Ry(DrawExpr),
    Rz(DrawExpr),
    // === Set scale ===
    Sm(DrawExpr),
    Sa(DrawExpr),
    // === Set ∪ reflected set ===
    Mirror(Axis),
    // === Add noise / sampled displacement ===
    Jitter(DrawExpr),
    Modulate(Field, DrawExpr),
    // === Per-emit color ===
    /// Multiply every emit's color by `(r, g, b, a)`. Composes with the
    /// brush's palette/gradient — instance color multiplies vertex color in
    /// the shader, so `Tint` brings out or knocks back channels rather than
    /// choosing colors. Args are expressions: reference `Stroke` to ramp the
    /// tint along an expanded stroke (`Tint(1, 1 - Stroke*0.5, 1 - Stroke*0.5)`
    /// reddens toward the stroke's end), or note fields for per-note shifts.
    /// Alpha defaults to `1.0` when omitted.
    Tint { r: DrawExpr, g: DrawExpr, b: DrawExpr, a: DrawExpr },
    // === Expansion: 1 emit → many ===
    /// Replace each emit with `n` samples linearly from `Here` to `Anchor`.
    Lerp { to: Anchor, n: DrawExpr },
    /// Replicate the set `n` times, accumulating `Increment` per copy.
    Spawn { n: DrawExpr, by: Increment },
    /// Temporal copies — pushes to the deferred-spawn lane.
    Echo { n: DrawExpr, dt: DrawExpr, decay: Option<DrawExpr> },
    /// Spread the set's emits linearly in time over `dur`.
    Stagger { over: DrawExpr },
    /// Release `n` emits over `dur` with progressive velocity offset.
    Drag { over: DrawExpr, n: DrawExpr, dv: DrawExpr },

    // === Recursion ===
    /// For each emit in the current set, run `sub` rebound to that emit's
    /// position. Union the results.
    Each(Box<DrawPipeline>),

    // === Higher-order combinators (conditional sub-pipeline application) ===
    /// `Every(n, sub)` — apply `sub` to the emit set only on notes where
    /// `count % n == 0`; pass through otherwise. Periodic accents.
    Every(DrawExpr, Box<DrawPipeline>),
    /// `Sometimes(p, sub)` — apply `sub` with probability `p` (0–1), decided
    /// by a per-note deterministic hash (stable across frames, not per-frame
    /// flicker). `Sometimes(0.3, Jitter(0.1))` roughens ~30% of notes.
    Sometimes(DrawExpr, Box<DrawPipeline>),

    /// Length multiply — scales each emit's `time_offset` by N.
    /// Audio-aligned: `Lm` is the universal "scale temporal extent"
    /// op. A draw chain that ends in `Stagger(over: 1)` puts 5 emits
    /// across 1 second; appending `| Lm 2` stretches that to 2
    /// seconds (the emits still land in the same proportional spots,
    /// just slower). Multiplicative, composes with anything that
    /// produces time-offset emits (`Echo`, `Stagger`, `Drag`).
    Lm(DrawExpr),

    // === Universal gap markers (lifted from audio) ===
    /// Identity / pass-through. Leaves the emit set unchanged. Useful as
    /// a slot marker in cross-DSL ModBy chains.
    AsIs,
    /// Kill — empties the emit set so this draw produces no marks this
    /// phase. `None | Lm 3` in a draw ModBy = "no emits for 3s, then
    /// resume." Source keyword `None`; AST variant `Mute` for parser
    /// hygiene.
    Mute,

    // === Composition primitives (mirror warp + audio) ===
    /// Time-multiplexed sub-pipelines. Each phase runs for its `length`
    /// seconds; cycle repeats, gated by the note's composition time
    /// (`sim_time`). Form: `Seq [ops | Lm m, ops | Lm n, …]`. Items
    /// default to `Lm 1` if no explicit length.
    Seq { phases: Vec<DrawPhase> },
    /// Parallel sub-pipelines whose emit sets union. Each layer starts
    /// from the current set, runs independently, then results are
    /// concatenated. Form: `Overlay [ops_chain, ops_chain, …]`.
    Overlay { layers: Vec<Vec<DrawOp>> },
}

/// One phase of a draw `Seq`. Same shape as warp `SeqPhase`.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawPhase {
    pub ops: Vec<DrawOp>,
    pub length: DrawExpr,
}

/// A draw pipeline: a generator (`Point`) followed by a list of `DrawOp`s
/// applied left-to-right.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawPipeline {
    pub ops: Vec<DrawOp>,
}

/// A top-level `draw NAME = { … }` definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawDef {
    pub name: String,
    pub pipeline: DrawPipeline,
}
