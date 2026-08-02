//! Strip `canvas NAME = { … }` blocks from .socool source before the audio
//! grammar sees them, returning the parsed defs.
//!
//! Same scan-and-blank shape as `light::preprocess::extract_lights`. A canvas,
//! like a light, is a fact about the piece rather than a program: no pipeline,
//! no chain, no attachment. Only `Paint(name)` refers to one, and that lives
//! inside a warp body which has already been extracted by the time this runs.
//!
//! Blocks are replaced BYTE-FOR-BYTE with spaces (newlines kept) so every
//! downstream byte offset — the promote pass's source scan above all — stays
//! aligned with the original file.

use crate::canvas::ast::CanvasDef;
use crate::canvas::canvas_grammar::FieldsParser;
use crate::dsl_extract::{find_matching_brace, is_ident_byte, matches_keyword, skip_ws};
use crate::dsl_parse_error::DslParseError;

#[derive(Debug)]
pub struct CanvasPreprocessed {
    /// Source with every `canvas NAME = { … }` block blanked.
    pub stripped: String,
    /// Parsed canvas defs, in source order.
    pub canvases: Vec<CanvasDef>,
}

#[derive(Debug)]
pub enum CanvasPreprocessError {
    UnbalancedBraces { start: usize },
    Parse { name: String, err: DslParseError },
}

impl CanvasPreprocessError {
    pub fn display(&self, quiet: bool) {
        if quiet {
            return;
        }
        match self {
            CanvasPreprocessError::Parse { name, err } => {
                eprintln!("[canvas] inside `canvas {} = {{ … }}`:", name);
                err.display(false);
            }
            CanvasPreprocessError::UnbalancedBraces { start } => {
                eprintln!("[canvas] unbalanced braces starting at byte {}", start)
            }
        }
    }
}

impl std::fmt::Display for CanvasPreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasPreprocessError::Parse { name, .. } => {
                write!(f, "canvas `{}` failed to parse", name)
            }
            CanvasPreprocessError::UnbalancedBraces { start } => {
                write!(f, "unbalanced braces starting at byte {}", start)
            }
        }
    }
}

pub fn extract_canvases(source: &str) -> Result<CanvasPreprocessed, CanvasPreprocessError> {
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut canvases: Vec<CanvasDef> = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // `canvas` at a word boundary — an audio def called `canvassing` must
        // survive untouched, and so must the many pieces with a local
        // `warp canvas = { … }`, which does not start with the bare keyword.
        if !matches_keyword(bytes, i, b"canvas") {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        let block_start = i;
        let mut j = skip_ws(bytes, i + 6);
        let name_start = j;
        while j < bytes.len() && is_ident_byte(bytes[j]) {
            j += 1;
        }
        if j == name_start {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let name = source[name_start..j].to_string();

        let k = skip_ws(bytes, j);
        if k >= bytes.len() || bytes[k] != b'=' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let k = skip_ws(bytes, k + 1);
        if k >= bytes.len() || bytes[k] != b'{' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        let body_start = k + 1;
        let body_end = match find_matching_brace(bytes, k) {
            Some(e) => e,
            None => return Err(CanvasPreprocessError::UnbalancedBraces { start: block_start }),
        };
        let body = &source[body_start..body_end];
        let fields = FieldsParser::new()
            .parse(body)
            .map_err(|e| CanvasPreprocessError::Parse {
                name: name.clone(),
                err: DslParseError::from_lalrpop("canvas", e, body),
            })?;
        canvases.push(CanvasDef::from_fields(name, fields));

        let block_end = body_end + 1;
        for p in block_start..block_end {
            out.push(if bytes[p] == b'\n' { b'\n' } else { b' ' });
        }
        i = block_end;
    }

    Ok(CanvasPreprocessed {
        stripped: String::from_utf8(out).expect("canvas stripper preserves UTF-8"),
        canvases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl_expr::Expr;

    #[test]
    fn extracts_a_surface_and_keeps_the_audio() {
        let src = "canvas linen = { ground: #f2ece0, tooth: 0.16 }\nx = { Fm 1 }";
        let pre = extract_canvases(src).unwrap();
        assert_eq!(pre.canvases.len(), 1);
        let c = &pre.canvases[0];
        assert_eq!(c.name, "linen");
        assert_eq!(c.ground.as_deref(), Some("#f2ece0"));
        assert_eq!(c.tooth, Some(Expr::Lit(0.16)));
        // The audio source keeps its bytes and its lines.
        assert_eq!(pre.stripped.len(), src.len());
        assert!(pre.stripped.contains("x = { Fm 1 }"));
        assert!(!pre.stripped.contains("linen"));
    }

    #[test]
    fn inner_brace_bags_are_order_free_and_partial() {
        let pre = extract_canvases("canvas p = { absorb: { neutral: 0.3, density: 9 } }").unwrap();
        let a = pre.canvases[0].absorb.clone().unwrap();
        assert_eq!(a.density, Expr::Lit(9.0));
        assert_eq!(a.neutral, Expr::Lit(0.3));
        // Unwritten knobs take the default rather than zero.
        assert_eq!(a.chroma, Expr::DefaultLit(crate::canvas::ast::defaults::ABSORB_CHROMA));
    }

    #[test]
    fn an_empty_body_is_a_usable_ground() {
        let pre = extract_canvases("canvas bare = { }").unwrap();
        let c = &pre.canvases[0];
        assert_eq!(c.name, "bare");
        assert!(c.ground.is_none());
        assert!(c.absorb.is_none());
    }

    #[test]
    fn relief_and_reflect_name_a_light() {
        let pre = extract_canvases(
            "canvas gilt = { relief: { depth: 6, light: studio }, reflect: { gain: 0.8, sharp: 60, light: studio } }",
        )
        .unwrap();
        let c = &pre.canvases[0];
        assert_eq!(c.relief.clone().unwrap().light.as_deref(), Some("studio"));
        let r = c.reflect.clone().unwrap();
        assert_eq!(r.gain, Expr::Lit(0.8));
        assert_eq!(r.light.as_deref(), Some("studio"));
    }

    #[test]
    fn a_local_warp_named_canvas_is_left_alone() {
        // Six pieces ship `warp canvas = { … }`. The keyword only binds at the
        // start of a def, so those must survive this pass untouched.
        let src = "warp canvas = { Scene }";
        let pre = extract_canvases(src).unwrap();
        assert!(pre.canvases.is_empty());
        assert_eq!(pre.stripped, src);
    }

    #[test]
    fn a_def_merely_starting_with_canvas_is_left_alone() {
        let src = "canvassing = { Fm 1 }";
        let pre = extract_canvases(src).unwrap();
        assert!(pre.canvases.is_empty());
        assert_eq!(pre.stripped, src);
    }
}
