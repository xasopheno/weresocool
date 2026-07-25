#[macro_use]
extern crate lalrpop_util;
// ── kintaro-DSL front end ──────────────────────────────────────────────
// The extended-language layer that runs BEFORE the socool grammar: `use`
// imports, parameterized defs, warp/draw/surface block extraction (each with
// its own lalrpop grammar), palette hoisting, and the shared expr/scanning
// helpers. Text in, text + ASTs out — the visual codegen for these ASTs
// lives in kintaro (bevy side), same split as `wgsl_dsl` below.
pub mod draw;
pub mod dsl_expr;
pub mod dsl_extract;
pub mod dsl_imports;
pub mod dsl_compose;
pub mod dsl_let;
pub mod dsl_params;
pub mod dsl_parse_error;
pub mod palette;
pub mod surface_dsl;
pub mod layer;
pub mod warp;

pub mod error_handling;
/// Re-exported so hosts that only depend on the parser (kintaro) can offer
/// the same did-you-mean suggestions the audio front end does.
pub use weresocool_error::nearest_names;
pub mod float_to_rational;
pub mod imports;
pub mod indices;
#[allow(clippy::all)]
pub mod parser;
pub mod tests;
pub mod wgsl_dsl;

pub use self::parser::{
    filename_to_vec_string, parse_file, parse_for_format, parse_to_raw_ast, process_wgsl_blocks, process_wgsl_blocks_with_validation,
    FormatParseResult, Init, ParsedComposition,
};
pub use self::wgsl_dsl::compile_dsl_to_wgsl;
