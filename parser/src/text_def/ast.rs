//! `text NAME = { "STRING", … }` — words the composition can be written into.
//!
//! A def, like `light` and `color`, for the same reason: it has a name and
//! several faces. Where the name appears picks which one —
//!
//!   `| Into(secret, t)` in a draw   the marks migrate into the letterforms
//!   `| Mask(secret)` in a warp      (not built) the paint shows only there
//!
//! The geometry is a STROKE alphabet, not an outline one: every glyph is a
//! handful of polylines, the way a pen plotter draws. That is not the cheap
//! choice, it is the right one — the marks in this language ARE lines, and a
//! mark landing on a stroke is handwriting. A mark landing on a TrueType
//! outline is either crawling a boundary (which reads as an outline, not a
//! letter) or filling an interior, which needs hundreds of marks to read at
//! all. The letterforms live in the visual host (`kintaro::text`); this half
//! only parses the declaration.

use crate::dsl_expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub struct TextDef {
    pub name: String,
    /// What it says. Uppercase Latin, digits and a little punctuation; the
    /// stroke alphabet folds lowercase up rather than refusing it.
    pub string: String,
    /// Cap height, in frame units. `None` → 0.3.
    pub size: Option<Expr>,
    /// Where the string's LEFT BASELINE sits, in frame fractions — the same
    /// space `Xf`/`Yf` place a mark in, so text is placed the way everything
    /// else is placed.
    pub at: Option<(Expr, Expr)>,
    /// Extra space between letters, as a fraction of cap height. `None` → 0.
    pub tracking: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextField {
    Size(Expr),
    At(Expr, Expr),
    Tracking(Expr),
}

impl TextDef {
    pub fn from_fields(name: String, string: String, fields: Vec<TextField>) -> Self {
        let mut out = TextDef { name, string, size: None, at: None, tracking: None };
        for f in fields {
            match f {
                TextField::Size(e) => out.size = Some(e),
                TextField::At(x, y) => out.at = Some((x, y)),
                TextField::Tracking(e) => out.tracking = Some(e),
            }
        }
        out
    }
}
