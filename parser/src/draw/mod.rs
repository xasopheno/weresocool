//! Draw DSL — parsing + extraction half (bevy-free).
//!
//! Same split as `crate::warp`: preprocess extracts `draw NAME = { … }`
//! blocks and inline `| name(args)` attachments, the lalrpop parser produces
//! the AST. The visual half (`compile.rs` — lowering to brush emit programs)
//! lives in the kintaro crate.

pub mod ast;
pub mod parser_lalrpop;
pub mod preprocess;

// The lalrpop-generated parser for the draw DSL.
// Grammar source: `src/draw/draw_grammar.lalrpop` (compiled by build.rs).
pub mod grammar_helpers;
lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub draw_grammar,
    "/draw/draw_grammar.rs"
);

/// Prefix for the synthetic `#__kdraw_<name>` tags preprocess splices into op
/// pipelines so the compiled draw routing (kintaro's `draw::compile`) can find
/// which voice carries which stroke. Lives here — the tag is written during
/// extraction — and is re-exported by kintaro's draw module for the compiler.
pub const TAG_PREFIX: &str = "kdraw_";

pub use ast::*;
pub use parser_lalrpop::{parse_pipeline_lalrpop as parse_pipeline, ParseError};
pub use preprocess::{extract_draws, Preprocessed, PreprocessError};
