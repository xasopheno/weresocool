//! `color NAME = [ … ]` — a palette with a name.
//!
//! `Color [#141014, #2b2118, …]` states six unrelated facts and cannot be
//! referred to: the identity of a palette is WHERE it was written, so the same
//! six hexes in two voices are two palettes and there is no way to say "these
//! two are the same family". A painter does not work that way — a limited
//! palette everything is mixed from is the normal case, not the exotic one.
//!
//! ```text
//! color zorn    = [#1a1a1a, #b8863b, #9c3a2e, #efe6d8]
//! color zorn_up = zorn | Reverse
//! color duo     = [zorn.1, complement(zorn.1)]
//!
//! bass = { … | zorn }        -- bare name, like `| wet` or `| form`
//! ```
//!
//! A named palette is a VALUE, not a shared runtime object: each use expands
//! to the literal list at that site, so every voice still mints its own brush
//! identity (see `ColorMap::insert_unique` — value-dedup once merged two
//! voices into one brush and lost the bass). What the name buys is one place
//! to edit and a way to derive.

/// One colour, before evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorExpr {
    /// `#rrggbb`, or a CSS / xkcd name.
    Literal(String),
    /// `zorn.2` — the nth colour of a named palette.
    Element(String, usize),
    /// `mix(a, b, t)` — perceptually, in OKLab.
    Mix(Box<ColorExpr>, Box<ColorExpr>, f32),
    /// `complement(c)` — the opposite hue, holding lightness. NOT `1 - c`.
    Complement(Box<ColorExpr>),
    /// `shade(c, t)` — toward black, the painter's sense of the word.
    Shade(Box<ColorExpr>, f32),
    /// `tint(c, t)` — toward white.
    Tint(Box<ColorExpr>, f32),
    /// `desaturate(c, t)` — toward neutral at the same lightness.
    Desaturate(Box<ColorExpr>, f32),
    /// `rotate(c, turns)` — turn the hue.
    Rotate(Box<ColorExpr>, f32),
}

/// A whole-palette transform. The operand is unambiguously a palette here,
/// which is why these live on the def side rather than in a voice chain: after
/// `Color [...]` in a chain the next `|` is an op on the VOICE, and `Reverse`
/// would have to mean something else entirely.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteOp {
    /// Turn the ramp end for end. The FIRST colour written lands at the lit
    /// end (Law 9), so this is how a palette written dark-first faces the
    /// light without retyping it — and retyping it would mint a second
    /// palette, which is the bug this whole def exists to remove.
    Reverse,
    /// Every colour's opposite hue.
    Complement,
    /// Pull the whole ramp toward neutral.
    Desaturate(f32),
    /// Turn every hue by the same amount.
    Rotate(f32),
}

/// The right-hand side of a `color` def: a list or another palette, then any
/// number of transforms.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteExpr {
    pub base: PaletteBase,
    pub ops: Vec<PaletteOp>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteBase {
    List(Vec<ColorExpr>),
    /// Another palette by name.
    Named(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorDef {
    pub name: String,
    pub expr: PaletteExpr,
}
