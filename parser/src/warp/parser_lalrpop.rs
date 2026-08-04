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

#[cfg(test)]
mod tests {
    use super::*;

    /// The block NAMES CHANNELS; it does not choose a buffer. `state` said
    /// otherwise, and 17 of the 36 defs that use it name only visible
    /// channels — so `fields` is the honest keyword and `state` is an alias
    /// those 17 keep working under.
    #[test]
    fn fields_and_state_are_the_same_block() {
        let a = parse_pipeline_with_state_lalrpop("fields { pigment: RGB, height: Sx } Prev")
            .expect("fields parses");
        let b = parse_pipeline_with_state_lalrpop("state { pigment: RGB, height: Sx } Prev")
            .expect("state still parses");
        assert_eq!(a, b);
        assert_eq!(a.0.len(), 2);
        assert_eq!(a.0[0], ("pigment".to_string(), Chan::Rgb));
        assert_eq!(a.0[1], ("height".to_string(), Chan::Sx));
    }

    /// A name is still a name — `fields` becoming a keyword must not stop a
    /// composer calling a channel `fields`.
    #[test]
    fn the_keyword_is_still_usable_as_a_field_name() {
        let (names, _) = parse_pipeline_with_state_lalrpop("fields { fields: Sx } Prev")
            .expect("parses");
        assert_eq!(names[0].0, "fields");
    }
}
