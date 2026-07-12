//! Warp DSL — parsing + extraction half (bevy-free).
//!
//! `.socool source → preprocess::extract_warps → parser → AST`. The visual
//! half (codegen → WGSL, lint, tweak, inspector) lives in the kintaro crate;
//! this half is enough to STRIP warp blocks from a composition so weresocool
//! can parse the audio, and to hand the parsed `WarpDef`s to that visual half.

pub mod ast;
pub mod preprocess;

// The lalrpop-generated parser for the warp DSL.
// Grammar source: `src/warp/warp_grammar.lalrpop` (compiled by build.rs).
pub mod grammar_helpers;
pub mod parser_lalrpop;
lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub warp_grammar,
    "/warp/warp_grammar.rs"
);

pub use ast::{Source, WarpBlendMode, WarpDef, WarpExpr, WarpOp, WarpPipeline};
pub use preprocess::{extract_warps, Preprocessed, PreprocessError};
