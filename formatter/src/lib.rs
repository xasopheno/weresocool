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
mod format_ast;

pub use config::FormatConfig;
pub use error::FormatError;
pub use format_ast::{FormatDef, FormatNode, FormatParseResult, FormatTerm, Span};

use weresocool_parser::{Init, ParsedComposition, parse_for_format};

/// Format source code string
pub fn format_source(source: &str, config: &FormatConfig) -> Result<String, FormatError> {
    let lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    let result = parse_for_format(lines)?;
    // Use original source for final output, but processed_source for span lookups
    // (spans are captured from processed source where WGSL is replaced with tokens)
    let span_source = result.processed_source.as_deref().unwrap_or(&result.source);
    Ok(format::format_composition_with_source(&result.composition, &result.source, span_source, config))
}

/// Format with access to original source (for span-based text extraction)
pub fn format_with_source(parsed: &ParsedComposition, source: &str, config: &FormatConfig) -> String {
    format::format_composition_with_source(parsed, source, source, config)
}

/// Format a parsed composition
pub fn format_parsed(parsed: &ParsedComposition, config: &FormatConfig) -> String {
    format::format_composition(parsed, config)
}

/// Format just an Init block
pub fn format_init(init: &Init, config: &FormatConfig) -> String {
    format::format_init_block(init, config)
}
