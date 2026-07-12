//! Surface DSL — parsing + extraction half (bevy-free).
//!
//! Strips `surface NAME = { … }` defs and `| surface NAME` clauses so
//! weresocool never sees them, and parses the pipelines to an AST. Mesh
//! generation (`mesh.rs`) is visual and lives in the kintaro crate.

pub mod ast;
pub mod parser;
pub mod preprocess;

pub use ast::*;
pub use parser::{parse_surface_pipeline, ParseError};
pub use preprocess::{extract_surfaces, SurfacePreprocessed, SurfacePreprocessError};
