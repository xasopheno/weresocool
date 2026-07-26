//! Thin wrapper for the lalrpop-generated draw grammar. Handles the one piece
//! of source rewriting the default lalrpop lexer can't: `τ` → `tau` (in
//! `grammar_helpers::preprocess`).

use crate::draw::ast::{DrawExpr, DrawPipeline};
use crate::draw::draw_grammar::{ExprParser, PipelineParser};
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

/// Parse a bare EXPRESSION with the shared expression grammar.
///
/// Law 4 says the visual DSLs share one expression language. warp and draw
/// share the AST, but brush `wgsl` captured its expressions with regexes,
/// because there was no way to call this grammar from outside a pipeline —
/// which is how wgsl ended up with its own dialect (quoted strings, an
/// uppercase-vs-lowercase lexer split, `Cycle` handled by a text rewrite).
/// This is that missing entry point.
pub fn parse_expr_lalrpop(src: &str) -> Result<DrawExpr, ParseError> {
    let rewritten = h::preprocess(src);
    ExprParser::new()
        .parse(&rewritten)
        .map_err(|e| ParseError::from_lalrpop("expression", e, &rewritten))
}
