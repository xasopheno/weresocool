//! `canvas NAME = { … }` — a surface is a def, not four positional numbers.
//!
//! The medium library used to spell a surface as `canvas(0.965, 6.5, 0.16,
//! 2.2)`, which says nothing about what any of those are, cannot grow a knob
//! without breaking every call, and gives the tweak panel four sliders all
//! labelled the same. As a def a surface gets a name (so a piece can say WHICH
//! ground it is painting on), named properties (so the numbers say what they
//! mean), and room to grow.
//!
//! Every property is optional and every default is the one measured on
//! `jdbeck_orbit` and `jdbeck_corridor`, so a bare `canvas foo = { }` is a
//! usable primed cloth rather than a blank.
//!
//! The consumer is kintaro's `Paint(name)` warp verb, which reads the pigment
//! field a `Lay` accumulated and runs these numbers over it.

use crate::dsl_expr::Expr;

/// The substrate's own texture — the weave of cloth, the fibre of paper.
/// `scale` is spatial frequency, `depth` how deep the trough, `slub` the
/// second, coarser frequency that keeps it from reading as graph paper.
///
/// `run` is THE DIRECTION THE FIBRE RUNS, as the ratio of the vertical scale
/// to the horizontal one. 1.0 is cloth: woven both ways, so the texture has no
/// grain. Paper is not cloth — it is pulp settled on a screen, and its fibres
/// lie down mostly one way — so `scale: 420, run: 0.21` samples 420 across and
/// 90 along, and the surface reads as a SHEET rather than as canvas. Four
/// pieces wrote that anisotropy by hand as `k_vnoise(vec2(uv.x * 420.0, uv.y *
/// 90.0))` and could not say it any other way.
#[derive(Debug, Clone, PartialEq)]
pub struct Weave {
    pub scale: Expr,
    pub depth: Expr,
    pub slub: Expr,
    pub run: Expr,
}

/// GRANULATION — pigment settling into the tooth instead of lying flat.
///
/// This is not the weave breaking the paint FILM (that is `tooth`, and it
/// happens at the surface). This is the pigment itself pooling in the low
/// places while the high places stay clear, so ABSORPTION varies at grain
/// scale — and it is the single thing that makes a wash read as watercolour
/// and not as airbrush. Which pigments granulate, and how coarsely, is a
/// property of the paper as much as of the paint.
///
/// `amount` is the swing about nominal: 0 is a perfectly even film, 0.56 is
/// the value `jdbeck_orbit` measured. `scale` and `slub` are its own two
/// frequencies, INDEPENDENT of the weave's — pigment settles at a coarser
/// scale than the fibre it settles into, and the pieces that hand-rolled this
/// used 210/47 against a 420/90 weave.
///
/// The mean is exactly 1, so turning `amount` up varies the absorption
/// without darkening or lightening the passage overall.
#[derive(Debug, Clone, PartialEq)]
pub struct Grain {
    pub scale: Expr,
    pub slub: Expr,
    pub amount: Expr,
}

/// Beer–Lambert: how the pigment takes light out of the ground.
///
/// `density` is the single most perceptually loaded number in a piece — at
/// x21 a tenth of a unit of paint reads 88% covered and every mark saturates
/// on landing; at x9 the same paint reads 59% and a mark keeps the gradient it
/// actually has. That range is the difference between a flat shape and a
/// brush.
///
/// `chroma` pulls a pigment back out from its own luminance, so piling paint
/// on DEEPENS the colour instead of drifting it toward white.
///
/// `neutral` is an achromatic absorption added on top, and it DEFAULTS TO
/// ZERO because it is a fossil. The old model normalised the pigment to its
/// brightest channel, so every colour came out with a channel at 1.0 and a
/// grey was indistinguishable from white — it had no complement left to
/// absorb with, and `neutral` was the patch that let it be dark anyway. Now
/// that the tint is real, `(1 - tint)` already absorbs correctly for greys AND
/// for black. Turning `neutral` back up is a deliberate effect (paint that
/// dirties as it thickens); leaving it on by default made white pigment
/// impossible, which is half the bug this whole channel was added to fix.
#[derive(Debug, Clone, PartialEq)]
pub struct Absorb {
    pub density: Expr,
    pub neutral: Expr,
    pub chroma: Expr,
    /// THE BLACK KNOB. How much pigment a mark carries per unit of COVERAGE
    /// when it has no brightness to declare it with.
    ///
    /// A subtractive canvas reads "how much paint" as `max(r,g,b)`, and
    /// #000000 has none — so black paint and bare ground are the same three
    /// numbers and no amount of tuning can separate them. Coverage is an
    /// independent witness that pigment is there, and this is how much it is
    /// worth. A coloured mark never touches it (its own brightness already
    /// exceeds the floor); only a dark one does.
    ///
    /// ZERO BY DEFAULT, and that default is load-bearing: at 0 a surface is
    /// exactly the canvas every existing palette was authored against. Turn
    /// it up to paint in blacks and greys.
    pub black: Expr,
}

/// A strike is wet, and a wet surface catches the light.
///
/// Absorption alone can only ever make a mark DARKER, so on a subtractive
/// ground a hard hit reads as a black blot — the opposite of landing hard.
/// Paint piled past what the ground can take stays on the surface. `knee` is
/// the load past which that happens: MEASURE IT, do not guess (run
/// `scripts/field_probe.py field <piece>` and put it between p90 and max).
/// `tint` is how far the highlight travels toward white — a highlight that
/// goes white everywhere is a blowout that throws away the colour which made
/// the mark worth looking at.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheen {
    pub knee: Expr,
    pub gain: Expr,
    pub tint: Expr,
}

/// Paint stands proud of the ground, and standing proud catches a raking
/// light. Coverage IS height — that is what the coverage channel bought — so
/// this needs no separate height field. `light` names a `light` def; absent,
/// the surface is lit flatly.
#[derive(Debug, Clone, PartialEq)]
pub struct Relief {
    pub depth: Expr,
    pub light: Option<String>,
}

/// A surface that returns light specularly rather than absorbing it — the
/// difference between gesso and gilding. `sharp` is the exponent on the
/// highlight: low is a broad sheen (brushed metal), high is a hard glint
/// (polished). Zero `gain` (the default) is an exact no-op, which is what
/// keeps every matte surface matte.
#[derive(Debug, Clone, PartialEq)]
pub struct Reflect {
    pub gain: Expr,
    pub sharp: Expr,
    pub light: Option<String>,
}

/// One declared surface.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasDef {
    pub name: String,
    /// `ground: #f6f2ea` — what the light comes back off where no paint sits.
    /// A hex literal or any CSS/xkcd colour name; resolved by the host.
    pub ground: Option<String>,
    pub weave: Option<Weave>,
    /// How much the weave breaks the paint film: where the substrate stands
    /// proud, less pigment sits in the trough.
    pub tooth: Option<Expr>,
    pub absorb: Option<Absorb>,
    /// Granulation: how unevenly the pigment settles into the tooth.
    pub grain: Option<Grain>,
    pub sheen: Option<Sheen>,
    /// Stretcher shading — a canvas is a physical object and its edges are
    /// where it is nailed to a frame. 0 is a surface with no edges.
    pub edge: Option<Expr>,
    pub relief: Option<Relief>,
    pub reflect: Option<Reflect>,
}

/// One `key: value` inside a canvas body. Flat and order-free, folded into
/// `CanvasDef` by [`CanvasDef::from_fields`] — the same brace-bag shape the
/// warp grammar and `light` use (DSL_STYLE Law 6).
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasField {
    Ground(String),
    Weave(Weave),
    Tooth(Expr),
    Absorb(Absorb),
    Grain(Grain),
    Sheen(Sheen),
    Edge(Expr),
    Relief(Relief),
    Reflect(Reflect),
}

/// The defaults, in one place so the grammar, the docs and the codegen cannot
/// drift apart. These are `canvas(0.965, 6.5, 0.16, 2.2)` — the preset proven
/// on jdbeck_corridor — written out with their names.
pub mod defaults {
    pub const GROUND: &str = "#f6f2ea";
    pub const WEAVE_SCALE: f32 = 300.0;
    pub const WEAVE_DEPTH: f32 = 0.055;
    pub const WEAVE_SLUB: f32 = 41.0;
    /// 1.0 — cloth, woven both ways. Below 1 the fibre runs across.
    pub const WEAVE_RUN: f32 = 1.0;
    pub const TOOTH: f32 = 0.16;
    pub const ABSORB_DENSITY: f32 = 6.5;
    /// Zero — see `Absorb`. The complement term already darkens greys.
    pub const ABSORB_NEUTRAL: f32 = 0.0;
    pub const ABSORB_CHROMA: f32 = 1.45;
    /// Zero — see `Absorb::black`. Off means "behave exactly as before".
    pub const ABSORB_BLACK: f32 = 0.0;
    /// Granulation is OFF by default: an even film is what a `canvas foo = {}`
    /// should be, and every existing surface was authored without one.
    pub const GRAIN_SCALE: f32 = 210.0;
    pub const GRAIN_SLUB: f32 = 47.0;
    pub const GRAIN_AMOUNT: f32 = 0.0;
    pub const SHEEN_KNEE: f32 = 2.2;
    pub const SHEEN_GAIN: f32 = 0.95;
    pub const SHEEN_TINT: f32 = 0.25;
    pub const EDGE: f32 = 0.14;
    /// Relief and reflect are OFF by default: a flat matte ground is the thing
    /// most pieces want, and both cost a gradient read.
    pub const RELIEF_DEPTH: f32 = 0.0;
    pub const REFLECT_GAIN: f32 = 0.0;
    pub const REFLECT_SHARP: f32 = 8.0;
}

impl CanvasDef {
    /// Fold parsed fields into a def. Later writes win, so a body that says
    /// `tooth:` twice takes the second — the same rule every other brace bag
    /// in the language follows.
    pub fn from_fields(name: String, fields: Vec<CanvasField>) -> Self {
        let mut out = CanvasDef {
            name,
            ground: None,
            weave: None,
            tooth: None,
            absorb: None,
            grain: None,
            sheen: None,
            edge: None,
            relief: None,
            reflect: None,
        };
        for f in fields {
            match f {
                CanvasField::Ground(c) => out.ground = Some(c),
                CanvasField::Weave(w) => out.weave = Some(w),
                CanvasField::Tooth(t) => out.tooth = Some(t),
                CanvasField::Absorb(a) => out.absorb = Some(a),
                CanvasField::Grain(g) => out.grain = Some(g),
                CanvasField::Sheen(s) => out.sheen = Some(s),
                CanvasField::Edge(e) => out.edge = Some(e),
                CanvasField::Relief(r) => out.relief = Some(r),
                CanvasField::Reflect(r) => out.reflect = Some(r),
            }
        }
        out
    }
}
