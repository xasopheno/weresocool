//! Helpers for the lalrpop-generated draw grammar.
//!
//! - Pre-processes the source so the byte-oriented lalrpop lexer can stay
//!   ASCII: `τ` (U+03C4) is replaced with the ASCII identifier `tau`.
//! - Folds an `Increment` chain (the pipe-separated form inside `Spawn(…,
//!   by: A | B | C)`) into either a single `Increment` or `Increment::Tuple`.
//! - Applies right-side scaling (`anchor * expr`) to `Anchor` values, matching
//!   the legacy parser's per-variant scaling semantics.

use crate::draw::ast::{Anchor, BinOp, DrawExpr, Increment};

/// Pre-process the raw draw block body: replace `τ` with `tau` so the rest of
/// the lexer stays ASCII. The two are equivalent identifiers in this DSL.
pub fn preprocess(src: &str) -> String {
    src.replace('τ', "tau")
}

/// Fold a `RATIONAL` token (`"a/b"`) to `a/b` as f32 — same arithmetic the hand
/// tokenizer used when it collapsed `INT/INT` into one rational literal.
pub fn parse_rational(s: &str) -> f32 {
    let mut it = s.split('/');
    let a: f32 = it.next().unwrap().parse().unwrap();
    let b: f32 = it.next().unwrap().parse().unwrap();
    a / b
}

/// Build an `Anchor` from a left value and a right-side scale factor.
/// Each anchor variant absorbs the scale differently — matches the hand
/// parser's `anchor_term` semantics. For `Add`/`Sub`/`Here`, scaling is a
/// silent no-op (the v1 grammar doesn't track scales for those).
pub fn scale_anchor(left: Anchor, scale: DrawExpr) -> Anchor {
    match left {
        Anchor::Direction(x, y, z) => Anchor::Direction(
            DrawExpr::Bin(BinOp::Mul, Box::new(x), Box::new(scale.clone())),
            DrawExpr::Bin(BinOp::Mul, Box::new(y), Box::new(scale.clone())),
            DrawExpr::Bin(BinOp::Mul, Box::new(z), Box::new(scale)),
        ),
        Anchor::Xa(e) => Anchor::Xa(DrawExpr::Bin(BinOp::Mul, Box::new(e), Box::new(scale))),
        Anchor::Ya(e) => Anchor::Ya(DrawExpr::Bin(BinOp::Mul, Box::new(e), Box::new(scale))),
        Anchor::Za(e) => Anchor::Za(DrawExpr::Bin(BinOp::Mul, Box::new(e), Box::new(scale))),
        other => other,
    }
}

/// Collapse a non-empty chain of `Increment` atoms into a single `Increment`.
/// `Spawn(…, by: A | B | C)` produces `Tuple([A, B, C])`; `by: A` produces
/// `A` directly.
pub fn fold_increment_chain(mut chain: Vec<Increment>) -> Increment {
    if chain.len() == 1 {
        chain.pop().unwrap()
    } else {
        Increment::Tuple(chain)
    }
}

/// If `ops` ends in a `DrawOp::Lm(e)`, peel it off and return its value
/// as the phase length. Otherwise return the ops untouched and `None`.
///
/// Used by `SeqPhaseItem` to give Seq phases their length without
/// needing a dedicated grammar rule for the trailing `| Lm N` — Lm is
/// always parsed as a regular `DrawOp::Lm`, and the Seq just looks at
/// the last op of each phase. Avoids the LR(1) conflict between
/// "phase suffix Lm" and "regular Lm op."
pub fn peel_trailing_lm(mut ops: Vec<crate::draw::ast::DrawOp>)
    -> (Vec<crate::draw::ast::DrawOp>, Option<crate::draw::ast::DrawExpr>)
{
    if matches!(ops.last(), Some(crate::draw::ast::DrawOp::Lm(_))) {
        if let Some(crate::draw::ast::DrawOp::Lm(e)) = ops.pop() {
            return (ops, Some(e));
        }
    }
    (ops, None)
}
