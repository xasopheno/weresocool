//! The `color NAME = [ … ]` def — palettes with names, and an algebra.
//!
//! Parse + expansion only. The output is ordinary `Color [ … ]` text, so every
//! downstream consumer — the audio grammar, the brush table, the DAW — sees
//! exactly the language it already saw.

pub mod ast;
pub mod expand;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub color_grammar,
    "/color_def/color_grammar.rs"
);

pub use ast::{ColorDef, ColorExpr, PaletteBase, PaletteExpr, PaletteOp};
pub use expand::{expand, expand_with_table, ColorDefError};
