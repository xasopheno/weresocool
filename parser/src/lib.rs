#[macro_use]
extern crate lalrpop_util;
pub mod error_handling;
pub mod float_to_rational;
pub mod imports;
pub mod indices;
#[allow(clippy::all)]
pub mod parser;
pub mod tests;
pub mod wgsl_dsl;

pub use self::parser::{
    filename_to_vec_string, parse_file, process_wgsl_blocks, process_wgsl_blocks_with_validation,
    Init, ParsedComposition,
};
pub use self::wgsl_dsl::compile_dsl_to_wgsl;
