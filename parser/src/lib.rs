#[macro_use]
extern crate lalrpop_util;
extern crate num_rational;
extern crate regex;
extern crate socool_ast;
pub mod error_handling;
pub mod float_to_rational;
pub mod imports;
#[allow(clippy::all)]
pub mod parser;

pub use self::float_to_rational::helpers::f32_to_rational;
pub use self::parser::{parse_file, Init, ParsedComposition};
