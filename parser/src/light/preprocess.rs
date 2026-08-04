//! Strip `light NAME = { … }` blocks from .socool source before the audio
//! grammar sees them, returning the parsed defs.
//!
//! Same scan-and-blank shape as `warp::preprocess::extract_warps`, minus
//! everything that made that one long: a light has no pipeline, no chain, no
//! attachment, and nothing refers to it from inside an audio def. It is the
//! smallest possible extractor, and deliberately so — a light is a fact about
//! the piece, not a program.
//!
//! Blocks are replaced BYTE-FOR-BYTE with spaces (newlines kept) so every
//! downstream byte offset — the promote pass's source scan above all — stays
//! aligned with the original file.

use crate::dsl_extract::scan_def_blocks;
use crate::dsl_parse_error::DslParseError;
use crate::light::ast::LightDef;
use crate::light::light_grammar::FieldsParser;

#[derive(Debug)]
pub struct LightPreprocessed {
    /// Source with every `light NAME = { … }` block blanked.
    pub stripped: String,
    /// Parsed light defs, in source order.
    pub lights: Vec<LightDef>,
}

#[derive(Debug)]
pub enum LightPreprocessError {
    UnbalancedBraces { start: usize },
    Parse { name: String, err: DslParseError },
}

impl LightPreprocessError {
    pub fn display(&self, quiet: bool) {
        if quiet {
            return;
        }
        match self {
            LightPreprocessError::Parse { name, err } => {
                eprintln!("[light] inside `light {} = {{ … }}`:", name);
                err.display(false);
            }
            LightPreprocessError::UnbalancedBraces { start } => {
                eprintln!("[light] unbalanced braces starting at byte {}", start)
            }
        }
    }
}

impl std::fmt::Display for LightPreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LightPreprocessError::Parse { name, .. } => {
                write!(f, "light `{}` failed to parse", name)
            }
            LightPreprocessError::UnbalancedBraces { start } => {
                write!(f, "unbalanced braces starting at byte {}", start)
            }
        }
    }
}

pub fn extract_lights(source: &str) -> Result<LightPreprocessed, LightPreprocessError> {
    let (stripped, blocks) = scan_def_blocks(source, "light")
        .map_err(|e| LightPreprocessError::UnbalancedBraces { start: e.0 })?;
    let mut lights = Vec::with_capacity(blocks.len());
    for b in blocks {
        let fields = FieldsParser::new().parse(&b.body).map_err(|e| {
            LightPreprocessError::Parse {
                name: b.name.clone(),
                err: DslParseError::from_lalrpop("light", e, &b.body),
            }
        })?;
        lights.push(LightDef::from_fields(b.name, fields));
    }
    Ok(LightPreprocessed { stripped, lights })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl_expr::Expr;

    #[test]
    fn extracts_a_directional_light() {
        let src = "light sun = { from: (-1, 1, 0), color: #ffd9a8, gain: 1.0 }\nx = { Fm 1 }";
        let pre = extract_lights(src).unwrap();
        assert_eq!(pre.lights.len(), 1);
        let l = &pre.lights[0];
        assert_eq!(l.name, "sun");
        assert_eq!(l.color.as_deref(), Some("#ffd9a8"));
        assert_eq!(l.gain, Some(Expr::Lit(1.0)));
        assert!(!l.ambient);
        assert!(l.from.is_some());
        // The audio source keeps its bytes and its lines.
        assert_eq!(pre.stripped.len(), src.len());
        assert!(pre.stripped.contains("x = { Fm 1 }"));
        assert!(!pre.stripped.contains("sun"));
    }

    #[test]
    fn ambient_needs_no_direction() {
        let pre = extract_lights("light sky = { ambient, color: skyblue, gain: 0.35 }").unwrap();
        let l = &pre.lights[0];
        assert!(l.ambient);
        assert!(l.from.is_none());
        assert_eq!(l.color.as_deref(), Some("skyblue"));
    }

    #[test]
    fn fields_are_expressions() {
        let pre = extract_lights("light sun = { from: (sin(clock * 0.1), 1, 0) }").unwrap();
        let (x, _, _) = pre.lights[0].from.clone().unwrap();
        assert!(matches!(x, Expr::Call(_, _)));
    }

    #[test]
    fn a_def_merely_starting_with_light_is_left_alone() {
        let src = "lighthouse = { Fm 1 }";
        let pre = extract_lights(src).unwrap();
        assert!(pre.lights.is_empty());
        assert_eq!(pre.stripped, src);
    }
}
