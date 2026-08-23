//! Camera DSL — `camera NAME = { from, look, up, fov, move }`.
//!
//! Parsing + extraction only (bevy-free), same split as `light`. The consumer
//! is kintaro: `camera::CameraRig` resolves the pose and folds the timeline
//! against the song clock, then writes the `PlayCamera` transform every frame.
//!
//! A declared camera OUTRANKS a saved `<piece>.socool.json`. That inverts the
//! rule the saved camera has always had, deliberately: a moving camera cannot
//! be meaningfully saved, and a piece stating its own viewpoint is a stronger
//! signal than a position someone flew to once. While a camera is declared the
//! json is not written back.

pub mod ast;
pub mod preprocess;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub camera_grammar,
    "/camera/camera_grammar.rs"
);

pub use ast::{CameraDef, CameraField, Move, MovePhase};
pub use preprocess::{extract_cameras, CameraPreprocessError, CameraPreprocessed};
