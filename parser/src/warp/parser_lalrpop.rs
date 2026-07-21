//! The warp parser: a thin wrapper invoking the lalrpop-generated grammar.
//! This is the sole warp parser (the old hand-written `parser.rs` was removed
//! once the differential test proved AST parity across every shipped `.socool`).

use crate::warp::ast::{Chan, WarpPipeline};
use crate::warp::warp_grammar::{OpsOnlyPipelineParser, PipelineParser, PipelineWithStateParser};

/// Warp parse errors go through `dsl_parse_error::DslParseError` for consistent
/// pretty source-context display across all kintaro DSLs.
pub use crate::dsl_parse_error::DslParseError as ParseError;

pub fn parse_pipeline_lalrpop(src: &str) -> Result<WarpPipeline, ParseError> {
    PipelineParser::new()
        .parse(src)
        .map_err(|e| ParseError::from_lalrpop("warp", e, src))
}

/// Parse a warp body that may begin with a `state { name: chan, ... }` block.
pub fn parse_pipeline_with_state_lalrpop(
    src: &str,
) -> Result<(Vec<(String, Chan)>, WarpPipeline), ParseError> {
    PipelineWithStateParser::new()
        .parse(src)
        .map_err(|e| ParseError::from_lalrpop("warp", e, src))
}

/// Parse an ops-only warp body (no source — implicit `Prev`). Used by the
/// inline `| warp { … }` form and by `background` bodies that start straight
/// at an op (`Raw { … } | Bloom …`).
pub fn parse_ops_only_lalrpop(src: &str) -> Result<WarpPipeline, ParseError> {
    OpsOnlyPipelineParser::new()
        .parse(src)
        .map_err(|e| ParseError::from_lalrpop("warp", e, src))
}
