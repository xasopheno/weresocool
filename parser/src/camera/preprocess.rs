//! Strip `camera NAME = { … }` blocks from .socool source before the audio
//! grammar sees them, returning the parsed defs.
//!
//! Same scan-and-blank shape as `light::preprocess::extract_lights`. A camera,
//! like a light, is a fact about the piece rather than a program: nothing
//! inside an audio def refers to one, so the extractor has nothing to reconnect
//! afterwards.
//!
//! Blocks are replaced BYTE-FOR-BYTE with spaces (newlines kept) so every
//! downstream byte offset stays aligned with the original file.

use crate::camera::ast::CameraDef;
use crate::camera::camera_grammar::FieldsParser;
use crate::dsl_extract::scan_def_blocks;
use crate::dsl_parse_error::DslParseError;

#[derive(Debug)]
pub struct CameraPreprocessed {
    /// Source with every `camera NAME = { … }` block blanked.
    pub stripped: String,
    /// Parsed camera defs, in source order.
    pub cameras: Vec<CameraDef>,
}

#[derive(Debug)]
pub enum CameraPreprocessError {
    UnbalancedBraces { start: usize },
    Parse { name: String, err: DslParseError },
}

impl CameraPreprocessError {
    pub fn display(&self, quiet: bool) {
        if quiet {
            return;
        }
        match self {
            CameraPreprocessError::Parse { name, err } => {
                eprintln!("[camera] inside `camera {} = {{ … }}`:", name);
                err.display(false);
            }
            CameraPreprocessError::UnbalancedBraces { start } => {
                eprintln!("[camera] unbalanced braces starting at byte {}", start)
            }
        }
    }
}

impl std::fmt::Display for CameraPreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraPreprocessError::Parse { name, .. } => {
                write!(f, "camera `{}` failed to parse", name)
            }
            CameraPreprocessError::UnbalancedBraces { start } => {
                write!(f, "unbalanced braces starting at byte {}", start)
            }
        }
    }
}

pub fn extract_cameras(source: &str) -> Result<CameraPreprocessed, CameraPreprocessError> {
    let (stripped, blocks) = scan_def_blocks(source, "camera")
        .map_err(|e| CameraPreprocessError::UnbalancedBraces { start: e.0 })?;
    let mut cameras = Vec::with_capacity(blocks.len());
    for b in blocks {
        let fields = FieldsParser::new()
            .parse(&b.body)
            .map_err(|e| CameraPreprocessError::Parse {
                name: b.name.clone(),
                err: DslParseError::from_lalrpop("camera", e, &b.body),
            })?;
        cameras.push(CameraDef::from_fields(b.name, fields));
    }
    Ok(CameraPreprocessed { stripped, cameras })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::ast::Move;
    use crate::dsl_expr::Expr;

    #[test]
    fn extracts_a_static_pose() {
        let src = "camera main = { from: (0, 0, 2.2), look: (0, 0, 0), fov: 60 }\nx = { Fm 1 }";
        let pre = extract_cameras(src).unwrap();
        assert_eq!(pre.cameras.len(), 1);
        let c = &pre.cameras[0];
        assert_eq!(c.name, "main");
        assert!(c.from.is_some());
        assert!(c.look.is_some());
        assert_eq!(c.fov, Some(Expr::Lit(60.0)));
        assert!(!c.moves_over_time());
        // The audio source keeps its bytes and its lines.
        assert_eq!(pre.stripped.len(), src.len());
        assert!(pre.stripped.contains("x = { Fm 1 }"));
        assert!(!pre.stripped.contains("2.2"));
    }

    #[test]
    fn a_clock_expression_is_a_moving_camera() {
        let pre =
            extract_cameras("camera main = { from: (sin(clock * 0.05) * 2.4, 0.3, 0) }").unwrap();
        assert!(pre.cameras[0].moves_over_time());
        let still = extract_cameras("camera main = { from: (2.4, 0.3, 0) }").unwrap();
        assert!(!still.cameras[0].moves_over_time());
    }

    #[test]
    fn a_timeline_of_moves() {
        let src = "camera main = { \
                   from: (0, 0, 2), \
                   move: Seq [ Hold | Lm 4, Dolly { by: -0.8 } | Lm 12, \
                               Orbit { turns: 1/4 } | Pedestal { by: 0.3 } | Lm 24 ] }";
        let c = &extract_cameras(src).unwrap().cameras[0];
        assert_eq!(c.moves.len(), 3);
        assert_eq!(c.moves[0].moves, vec![Move::Hold]);
        assert_eq!(c.moves[0].length, Expr::Lit(4.0));
        assert_eq!(c.moves[1].moves, vec![Move::Dolly(Expr::Bin(
            crate::dsl_expr::BinOp::Sub,
            Box::new(Expr::DefaultLit(0.0)),
            Box::new(Expr::Lit(0.8))
        ))]);
        // Two moves in one phase run together.
        assert_eq!(c.moves[2].moves.len(), 2);
        assert!(c.moves_over_time());
    }

    #[test]
    fn positional_and_brace_forms_agree() {
        let a = &extract_cameras("camera m = { move: Seq [ Dolly(-1) ] }").unwrap().cameras[0];
        let b = &extract_cameras("camera m = { move: Seq [ Dolly { by: -1 } ] }").unwrap().cameras[0];
        assert_eq!(a.moves, b.moves);
    }

    #[test]
    fn a_phase_without_lm_is_one_unit() {
        let c = &extract_cameras("camera m = { move: Seq [ Hold ] }").unwrap().cameras[0];
        assert_eq!(c.moves[0].length, Expr::DefaultLit(1.0));
    }

    #[test]
    fn fit_length_names_a_def() {
        let c = &extract_cameras("camera m = { move: Seq [ Orbit(1) ] | FitLength main }")
            .unwrap()
            .cameras[0];
        assert_eq!(c.fit.as_deref(), Some("main"));
    }

    #[test]
    fn a_def_merely_starting_with_camera_is_left_alone() {
        let src = "cameraman = { Fm 1 }";
        let pre = extract_cameras(src).unwrap();
        assert!(pre.cameras.is_empty());
        assert_eq!(pre.stripped, src);
    }
}
