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
use crate::dsl_extract::scan_def_blocks;
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
    let (stripped, blocks) = scan_def_blocks(source, "canvas")
        .map_err(|e| CanvasPreprocessError::UnbalancedBraces { start: e.0 })?;
    let mut canvases = Vec::with_capacity(blocks.len());
    for b in blocks {
        let fields = FieldsParser::new().parse(&b.body).map_err(|e| {
            CanvasPreprocessError::Parse {
                name: b.name.clone(),
                err: DslParseError::from_lalrpop("canvas", e, &b.body),
            }
        })?;
        canvases.push(CanvasDef::from_fields(b.name, fields));
    }
    Ok(CanvasPreprocessed { stripped, canvases })
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
    fn a_repeated_inner_key_is_last_wins_like_the_outer_bag() {
        let pre = extract_canvases("canvas p = { weave: { scale: 420, scale: 210 } }").unwrap();
        assert_eq!(pre.canvases[0].weave.clone().unwrap().scale, Expr::Lit(210.0));
    }

    #[test]
    fn a_key_word_is_still_usable_as_a_name() {
        // Seventeen words are keys in this grammar. Naming a light after one
        // of them used to be `UnrecognizedToken` — the whole piece refused to
        // load — because the two bare-name positions had no keyword fallback.
        let pre = extract_canvases(
            "canvas gilt = { reflect: { gain: 0.8, light: run }, ground: tan }",
        )
        .unwrap();
        let c = &pre.canvases[0];
        assert_eq!(c.reflect.clone().unwrap().light.as_deref(), Some("run"));
        assert_eq!(c.ground.as_deref(), Some("tan"));
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
    fn the_fibre_runs_one_way_and_the_pigment_granulates() {
        // The two properties four pieces needed and could not say: `run` is
        // the paper's fibre direction, `grain` is the pigment settling into
        // it. Both were `Raw { }` before there was anywhere to put them.
        let pre = extract_canvases(
            "canvas rag = { weave: { scale: 420, run: 90/420 }, grain: { scale: 210, slub: 47, amount: 0.56 } }",
        )
        .unwrap();
        let c = &pre.canvases[0];
        let w = c.weave.clone().unwrap();
        assert_eq!(w.scale, Expr::Lit(420.0));
        assert_eq!(w.run, Expr::Lit(90.0 / 420.0));
        let g = c.grain.clone().unwrap();
        assert_eq!(g.scale, Expr::Lit(210.0));
        assert_eq!(g.amount, Expr::Lit(0.56));
    }

    #[test]
    fn a_surface_that_says_neither_is_cloth_with_an_even_film() {
        // Both defaults are exact no-ops in the shader — `run: 1` samples the
        // noise square and `amount: 0` multiplies absorption by 1 — which is
        // what keeps every already-authored surface bit-identical.
        let pre = extract_canvases("canvas duck = { weave: { scale: 300 } }").unwrap();
        let c = &pre.canvases[0];
        assert_eq!(
            c.weave.clone().unwrap().run,
            Expr::DefaultLit(crate::canvas::ast::defaults::WEAVE_RUN)
        );
        assert!(c.grain.is_none());
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
    fn a_canvas_named_in_a_comment_is_prose_not_a_def() {
        // The medium library's own docs say this. A comment-blind scan reads
        // it as a def and dies on the ellipsis.
        let src = "-- shadow it by writing `canvas linen = { … }` in the piece.\nx = { Fm 1 }";
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
