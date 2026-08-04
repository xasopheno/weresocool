//! Strip `text NAME = { "…" }` blocks before any other DSL sees them.
//!
//! Same scan-and-blank shape as the light extractor, and for the same reason:
//! a text def is a leaf, nothing refers to one from inside a warp or a draw
//! body, and the audio path has no use for it. Blocks are replaced byte for
//! byte with spaces so every downstream byte-span stays aligned.

use crate::dsl_extract::scan_def_blocks;
use crate::dsl_parse_error::DslParseError;
use crate::text_def::ast::TextDef;
use crate::text_def::text_grammar::BodyParser;

#[derive(Debug)]
pub struct TextPreprocessed {
    pub stripped: String,
    pub texts: Vec<TextDef>,
}

#[derive(Debug)]
pub enum TextPreprocessError {
    UnbalancedBraces { start: usize },
    Parse { name: String, err: DslParseError },
}

impl TextPreprocessError {
    pub fn display(&self, quiet: bool) {
        if quiet { return; }
        match self {
            TextPreprocessError::Parse { name, err } => {
                eprintln!("[text] inside `text {} = {{ … }}`:", name);
                err.display(false);
            }
            TextPreprocessError::UnbalancedBraces { start } =>
                eprintln!("[text] unbalanced braces starting at byte {}", start),
        }
    }
}

impl std::fmt::Display for TextPreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextPreprocessError::Parse { name, .. } => write!(f, "text `{}` failed to parse", name),
            TextPreprocessError::UnbalancedBraces { start } =>
                write!(f, "unbalanced braces starting at byte {}", start),
        }
    }
}

pub fn extract_texts(source: &str) -> Result<TextPreprocessed, TextPreprocessError> {
    let (stripped, blocks) = scan_def_blocks(source, "text")
        .map_err(|e| TextPreprocessError::UnbalancedBraces { start: e.0 })?;
    let mut texts = Vec::with_capacity(blocks.len());
    for b in blocks {
        let (string, fields) = BodyParser::new().parse(&b.body).map_err(|e| {
            TextPreprocessError::Parse {
                name: b.name.clone(),
                err: DslParseError::from_lalrpop("text", e, &b.body),
            }
        })?;
        texts.push(TextDef::from_fields(b.name, string, fields));
    }
    Ok(TextPreprocessed { stripped, texts })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_a_string_and_its_placement() {
        let src = "text secret = { \"REMEMBER\", size: 0.42, at: (0, 0.1) }\nx = { Fm 1 }";
        let pre = extract_texts(src).unwrap();
        assert_eq!(pre.texts.len(), 1);
        assert_eq!(pre.texts[0].name, "secret");
        assert_eq!(pre.texts[0].string, "REMEMBER");
        assert!(pre.texts[0].size.is_some() && pre.texts[0].at.is_some());
        assert_eq!(pre.stripped.len(), src.len());
        assert!(pre.stripped.contains("x = { Fm 1 }"));
    }

    #[test]
    fn the_string_is_the_only_required_term() {
        let pre = extract_texts("text a = { \"HI\" }").unwrap();
        assert_eq!(pre.texts[0].string, "HI");
        assert!(pre.texts[0].size.is_none());
    }

    #[test]
    fn a_def_merely_starting_with_text_is_left_alone() {
        let src = "texture = { Fm 1 }";
        assert_eq!(extract_texts(src).unwrap().stripped, src);
        assert!(extract_texts(src).unwrap().texts.is_empty());
    }
}
