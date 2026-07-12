//! The warp parser: a thin wrapper invoking the lalrpop-generated grammar.
//! This is the sole warp parser (the old hand-written `parser.rs` was removed
//! once the differential test proved AST parity across every shipped `.socool`).

use crate::warp::ast::WarpPipeline;
use crate::warp::warp_grammar::PipelineParser;

/// Warp parse errors go through `dsl_parse_error::DslParseError` for consistent
/// pretty source-context display across all kintaro DSLs.
pub use crate::dsl_parse_error::DslParseError as ParseError;

pub fn parse_pipeline_lalrpop(src: &str) -> Result<WarpPipeline, ParseError> {
    PipelineParser::new()
        .parse(src)
        .map_err(|e| ParseError::from_lalrpop("warp", e, src))
}
