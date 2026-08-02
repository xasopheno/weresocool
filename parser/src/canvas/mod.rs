//! Canvas DSL — `canvas NAME = { ground, weave, tooth, absorb, sheen, edge,
//! relief, reflect }`.
//!
//! Parsing + extraction only (bevy-free). The consumer is in kintaro: the
//! `Paint(name)` warp verb bakes a named surface into the generated shader and
//! runs it over the pigment field a `Lay` accumulated.
//!
//! A canvas is the MATERIAL — how a ground takes paint and returns light. It
//! is not `surface`, which is the GEOMETRY the finished image hangs in. You
//! paint on a canvas; the canvas hangs on a surface.

pub mod ast;
pub mod helpers;
pub mod preprocess;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub canvas_grammar,
    "/canvas/canvas_grammar.rs"
);

pub use ast::{Absorb, CanvasDef, CanvasField, Reflect, Relief, Sheen, Weave};
pub use preprocess::{extract_canvases, CanvasPreprocessError, CanvasPreprocessed};
