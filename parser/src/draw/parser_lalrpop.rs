//! Thin wrapper for the lalrpop-generated draw grammar. Handles the one piece
//! of source rewriting the default lalrpop lexer can't: `τ` → `tau` (in
//! `grammar_helpers::preprocess`).

use crate::draw::ast::DrawPipeline;
use crate::draw::draw_grammar::PipelineParser;
use crate::draw::grammar_helpers as h;

/// Draw parse errors go through `dsl_parse_error::DslParseError` for consistent
/// pretty source-context display across all kintaro DSLs.
pub use crate::dsl_parse_error::DslParseError as ParseError;

pub fn parse_pipeline_lalrpop(src: &str) -> Result<DrawPipeline, ParseError> {
    let rewritten = h::preprocess(src);
    PipelineParser::new()
        .parse(&rewritten)
        .map_err(|e| ParseError::from_lalrpop("draw", e, &rewritten))
}
