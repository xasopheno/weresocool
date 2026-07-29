//! Light DSL — `light NAME = { from, color, gain, ambient }`.
//!
//! Parsing + extraction only (bevy-free). The consumers are in kintaro:
//! `light::LightTable` resolves the expressions, the instancing shader bakes
//! the table, and `Gradient(name)` / `Shade(name)` pick one by name. See
//! `crates/kintaro/docs/COLOR.md` §5.

pub mod ast;
pub mod preprocess;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub light_grammar,
    "/light/light_grammar.rs"
);

pub use ast::{LightDef, LightField};
pub use preprocess::{extract_lights, LightPreprocessError, LightPreprocessed};
