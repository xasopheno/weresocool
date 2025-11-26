//! WereSoCool source code formatter
//!
//! This crate provides formatting capabilities for `.socool` files.
//! It parses source code into an AST and then pretty-prints it back
//! to a canonical format.
//!
//! ## Note on Canonicalization
//! The formatter normalizes syntax to canonical forms:
//! - `Tm` and `Fm` both become `Fm` (frequency multiplier)
//! - `Ta` and `Fa` both become `Fa` (frequency add)
//! - `O[...]` becomes `Overlay [...]`
//! - Point notation `(f, a, g, p)` expands to explicit ops

mod config;
mod error;
mod format;

pub use config::FormatConfig;
pub use error::FormatError;

use weresocool_parser::{Init, ParsedComposition, parse_to_raw_ast};

/// Format source code string
pub fn format_source(source: &str, config: &FormatConfig) -> Result<String, FormatError> {
    let lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    let parsed = parse_to_raw_ast(lines)?;
    Ok(format_parsed(&parsed, config))
}

/// Format a parsed composition
pub fn format_parsed(parsed: &ParsedComposition, config: &FormatConfig) -> String {
    format::format_composition(parsed, config)
}

/// Format just an Init block
pub fn format_init(init: &Init, config: &FormatConfig) -> String {
    format::format_init_block(init, config)
}
