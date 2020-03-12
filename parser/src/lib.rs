#[macro_use]
extern crate lalrpop_util;
pub mod error_handling;
pub mod float_to_rational;
pub mod imports;
pub mod indices;
#[allow(clippy::all)]
pub mod parser;

pub use self::parser::{filename_to_vec_string, parse_file, Init, ParsedComposition};
