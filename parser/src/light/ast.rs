//! `light NAME = { … }` — a light is a def, not a setting.
//!
//! The header's `light: (x, y, z)` said a piece has ONE light and it never
//! moves and it has no colour. All three of those were consequences of where
//! it was written, not decisions anyone made. As a def a light gets a name (so
//! a draw can say WHICH light its gradient follows), a colour (so shadow can
//! be a colour rather than a darkness — the oldest trick in painting), and
//! fields that are expressions (so it can move).
//!
//! See `crates/kintaro/docs/COLOR.md` §5. The consumers live in kintaro:
//! `light::LightTable` resolves these, the instancing shader bakes them, and
//! `Gradient(name)` / `Shade(name)` pick one by name.

use crate::dsl_expr::Expr;

/// One declared light.
///
/// Every field is optional; the defaults are the ones a painter would assume —
/// a light with no direction is ambient, a light with no colour is white, a
/// light with no gain is at full strength. Direction is whence: where the
/// light falls FROM, the same convention the header always used.
#[derive(Debug, Clone, PartialEq)]
pub struct LightDef {
    pub name: String,
    /// `from: (x, y, z)`. Expressions so a light can travel:
    /// `from: (sin(clock * 0.1), 1, 0)`.
    pub from: Option<(Expr, Expr, Expr)>,
    /// `color: #ffd9a8` or a CSS/xkcd name. Resolved to linear rgb by the host.
    pub color: Option<String>,
    /// `gain: 0.35` — how much of it there is. Absent means "take it from the
    /// length of `from`", which is how the header light has always worked.
    pub gain: Option<Expr>,
    /// The bare marker `ambient`: sky, bounce, the thing that fills a shadow.
    /// Ambient lights have no direction — a `from` alongside it is ignored.
    pub ambient: bool,
}

/// One `key: value` (or bare marker) inside a light body. Flat, order-free,
/// resolved into `LightDef` by [`LightDef::from_fields`] — the same shape the
/// warp grammar uses for its named args.
#[derive(Debug, Clone, PartialEq)]
pub enum LightField {
    From(Expr, Expr, Expr),
    Color(String),
    Gain(Expr),
    Ambient,
}

impl LightDef {
    /// Fold parsed fields into a def. Later writes win, so a body that says
    /// `color:` twice takes the second — the same rule the rest of the
    /// brace-bag args follow.
    pub fn from_fields(name: String, fields: Vec<LightField>) -> Self {
        let mut out = LightDef {
            name,
            from: None,
            color: None,
            gain: None,
            ambient: false,
        };
        for f in fields {
            match f {
                LightField::From(x, y, z) => out.from = Some((x, y, z)),
                LightField::Color(c) => out.color = Some(c),
                LightField::Gain(g) => out.gain = Some(g),
                LightField::Ambient => out.ambient = true,
            }
        }
        out
    }
}
