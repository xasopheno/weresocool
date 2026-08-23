//! `camera NAME = { … }` — the viewpoint is part of the composition.
//!
//! Until now the camera came from three places, none of them the piece: a
//! `<name>.socool.json` saved by flying around in watch, the header's `w:`/`h:`
//! by way of `camera_for_frame`, or a resolution-derived default. So a piece
//! could say where its edges are and where its light falls from but not where
//! it is seen from, and nothing about the view could change while it played.
//!
//! Two ways to say it, and they compose:
//!
//!   THE POSE — `from` / `look` / `up` / `fov`, every one an expression, so
//!   `clock` moves them exactly as it already moves a light. This is POV-Ray's
//!   `camera { location … look_at … angle … }`, and it is enough on its own:
//!   `from: (sin(clock * 0.05) * 2.4, 0.3, cos(clock * 0.05) * 2.4)` orbits.
//!
//!   THE MOVES — a timed `Seq` of the words cinema already uses for this:
//!   dolly, truck, pedestal, pan, tilt, roll, orbit, zoom. Each is a delta
//!   applied over its `Lm` window, relative to the pose. Trigonometry can
//!   express any of them; none of it says what it means.
//!
//! Rotations are in TURNS, not radians — `Pan { by: 1/4 }` is a quarter turn,
//! matching `Rz(1/4)` in the draw DSL.
//!
//! The consumer is kintaro's `camera::CameraRig`, which resolves these against
//! the `SimTime` song clock every frame and writes the `PlayCamera` transform.

use crate::dsl_expr::Expr;

/// One declared camera.
///
/// Every field is optional and the defaults are the ones already in force, so
/// `camera main = { }` renders exactly as no camera at all: the pose falls back
/// to whatever the piece would otherwise have used, and an empty timeline holds
/// it there.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraDef {
    pub name: String,
    /// Where the camera stands.
    pub from: Option<(Expr, Expr, Expr)>,
    /// What it points at. With `from` but no `look`, it keeps looking at the
    /// origin, which is where every piece so far has put its subject.
    pub look: Option<(Expr, Expr, Expr)>,
    /// Which way is up. Absent, +Y.
    pub up: Option<(Expr, Expr, Expr)>,
    /// Vertical field of view in degrees. Absent, the renderer's default 60.
    pub fov: Option<Expr>,
    /// The timeline, in order. Empty means the pose is the whole story.
    pub moves: Vec<MovePhase>,
    /// `Seq [ … ] | FitLength name` — stretch the timeline so it spans that
    /// def's length. Without it, phase lengths are in the piece's own `l`
    /// units and a timeline shorter than the piece simply ends, holding its
    /// final pose.
    pub fit: Option<String>,
}

/// One step of the timeline: the moves that run together, and how long they
/// take. `Dolly { by: -1 } | Pan { by: 1/8 } | Lm 4` is one phase.
#[derive(Debug, Clone, PartialEq)]
pub struct MovePhase {
    pub moves: Vec<Move>,
    /// Length in `l` units. Defaults to 1, same as a `Seq` phase everywhere else.
    pub length: Expr,
}

/// The vocabulary. Three translations, three rotations in place, one rotation
/// about the subject, and the one that is not a movement at all.
#[derive(Debug, Clone, PartialEq)]
pub enum Move {
    /// Along the view axis. Negative pulls back.
    Dolly(Expr),
    /// Sideways, perpendicular to view and up.
    Truck(Expr),
    /// Straight up, along `up`.
    Pedestal(Expr),
    /// Yaw, in place. Turns.
    Pan(Expr),
    /// Pitch, in place. Turns.
    Tilt(Expr),
    /// Roll about the view axis. Turns.
    Roll(Expr),
    /// Swing around `look`, keeping distance. Turns. The one move that is
    /// about the subject rather than about the camera.
    Orbit(Expr),
    /// Field of view, in degrees — the move that is not a move. A zoom and a
    /// dolly look different for exactly this reason, so they are two words.
    Zoom(Expr),
    /// Stay. A camera that never rests is unwatchable, and a timeline needs a
    /// way to say so rather than leaving gaps.
    Hold,
}

/// One `key: value` (or `move: Seq […]`) inside a camera body. Flat and
/// order-free, folded into `CameraDef` by [`CameraDef::from_fields`] — the same
/// brace-bag shape `light` and the warp named args use.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraField {
    From(Expr, Expr, Expr),
    Look(Expr, Expr, Expr),
    Up(Expr, Expr, Expr),
    Fov(Expr),
    Moves(Vec<MovePhase>, Option<String>),
}

impl CameraDef {
    /// Fold parsed fields into a def. Later writes win, so a body that says
    /// `from:` twice takes the second — the rule the other brace bags follow.
    pub fn from_fields(name: String, fields: Vec<CameraField>) -> Self {
        let mut out = CameraDef {
            name,
            from: None,
            look: None,
            up: None,
            fov: None,
            moves: Vec::new(),
            fit: None,
        };
        for f in fields {
            match f {
                CameraField::From(x, y, z) => out.from = Some((x, y, z)),
                CameraField::Look(x, y, z) => out.look = Some((x, y, z)),
                CameraField::Up(x, y, z) => out.up = Some((x, y, z)),
                CameraField::Fov(v) => out.fov = Some(v),
                CameraField::Moves(m, fit) => {
                    out.moves = m;
                    out.fit = fit;
                }
            }
        }
        out
    }

    /// Does anything about this camera change over time? A static camera can be
    /// written once at setup and never touched again; a moving one needs a
    /// system running every frame. Mirrors `Light::moves`.
    pub fn moves_over_time(&self) -> bool {
        !self.moves.is_empty()
            || tri_uses_clock(&self.from)
            || tri_uses_clock(&self.look)
            || tri_uses_clock(&self.up)
            || self.fov.as_ref().is_some_and(uses_clock)
    }
}

fn tri_uses_clock(t: &Option<(Expr, Expr, Expr)>) -> bool {
    t.as_ref()
        .is_some_and(|(x, y, z)| uses_clock(x) || uses_clock(y) || uses_clock(z))
}

/// Whether an expression reads the clock, walked structurally. Same shape as
/// `kintaro::light::uses_clock`; kept here so the parser can answer it without
/// the host.
pub fn uses_clock(e: &Expr) -> bool {
    match e {
        Expr::Clock => true,
        Expr::Bin(_, a, b) => uses_clock(a) || uses_clock(b),
        Expr::Call(_, args) => args.iter().any(uses_clock),
        Expr::Sin { freq, amp } => uses_clock(freq) || uses_clock(amp),
        _ => false,
    }
}
