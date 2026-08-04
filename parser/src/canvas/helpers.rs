//! Helpers for the lalrpop-generated canvas grammar.
//!
//! The inner brace bags (`absorb: { density: 9, neutral: 0.2 }`) are parsed
//! FLAT — a `Vec<(String, SubVal)>` — and sorted out here. Encoding optionality
//! in the grammar would mean one rule per subset of keys, which is the
//! combinatorial explosion `warp/grammar_helpers.rs` exists to avoid; this is
//! the same trick applied to the same problem.
//!
//! An unknown key WARNS and falls back to the default rather than failing. A
//! surface is a set of adjustments to a working ground, so a typo should cost
//! you one property, not the whole piece.

use crate::canvas::ast::{defaults, Absorb, Grain, Reflect, Relief, Sheen, Weave};
use crate::dsl_expr::Expr;

/// A value inside an inner brace bag: a number-ish expression, or the name of
/// a `light` def (`light: studio`).
#[derive(Debug, Clone, PartialEq)]
pub enum SubVal {
    Num(Expr),
    Name(String),
}

fn take(args: &mut Vec<(String, SubVal)>, key: &str) -> Option<SubVal> {
    args.iter()
        .position(|(k, _)| k == key)
        .map(|p| args.remove(p).1)
}

/// One numeric knob, or its default. A `light: foo` written where a number
/// belongs is a category error, so it warns and defaults too.
fn take_num(args: &mut Vec<(String, SubVal)>, key: &str, default: f32) -> Expr {
    match take(args, key) {
        Some(SubVal::Num(e)) => e,
        Some(SubVal::Name(n)) => {
            eprintln!("[canvas] `{key}:` wants a number, got the name `{n}` — using default");
            Expr::DefaultLit(default)
        }
        None => Expr::DefaultLit(default),
    }
}

fn take_light(args: &mut Vec<(String, SubVal)>) -> Option<String> {
    match take(args, "light") {
        Some(SubVal::Name(n)) => Some(n),
        Some(SubVal::Num(_)) => {
            eprintln!("[canvas] `light:` wants the name of a `light` def, got a number — ignoring");
            None
        }
        None => None,
    }
}

/// Warn about keys nobody consumed — a typo'd knob must not vanish silently.
fn warn_leftover(group: &str, args: &[(String, SubVal)], known: &[&str]) {
    for (k, _) in args {
        eprintln!(
            "[canvas] {group}: unknown key `{k}:` ignored (known: {})",
            known.join(", ")
        );
    }
}

pub fn weave_of(mut a: Vec<(String, SubVal)>) -> Weave {
    let w = Weave {
        scale: take_num(&mut a, "scale", defaults::WEAVE_SCALE),
        depth: take_num(&mut a, "depth", defaults::WEAVE_DEPTH),
        slub: take_num(&mut a, "slub", defaults::WEAVE_SLUB),
        run: take_num(&mut a, "run", defaults::WEAVE_RUN),
    };
    warn_leftover("weave", &a, &["scale", "depth", "slub", "run"]);
    w
}

pub fn absorb_of(mut a: Vec<(String, SubVal)>) -> Absorb {
    let v = Absorb {
        density: take_num(&mut a, "density", defaults::ABSORB_DENSITY),
        neutral: take_num(&mut a, "neutral", defaults::ABSORB_NEUTRAL),
        chroma: take_num(&mut a, "chroma", defaults::ABSORB_CHROMA),
        black: take_num(&mut a, "black", defaults::ABSORB_BLACK),
    };
    warn_leftover("absorb", &a, &["density", "neutral", "chroma", "black"]);
    v
}

pub fn grain_of(mut a: Vec<(String, SubVal)>) -> Grain {
    let v = Grain {
        scale: take_num(&mut a, "scale", defaults::GRAIN_SCALE),
        slub: take_num(&mut a, "slub", defaults::GRAIN_SLUB),
        amount: take_num(&mut a, "amount", defaults::GRAIN_AMOUNT),
    };
    warn_leftover("grain", &a, &["scale", "slub", "amount"]);
    v
}

pub fn sheen_of(mut a: Vec<(String, SubVal)>) -> Sheen {
    let v = Sheen {
        knee: take_num(&mut a, "knee", defaults::SHEEN_KNEE),
        gain: take_num(&mut a, "gain", defaults::SHEEN_GAIN),
        tint: take_num(&mut a, "tint", defaults::SHEEN_TINT),
    };
    warn_leftover("sheen", &a, &["knee", "gain", "tint"]);
    v
}

pub fn relief_of(mut a: Vec<(String, SubVal)>) -> Relief {
    let light = take_light(&mut a);
    let v = Relief {
        depth: take_num(&mut a, "depth", defaults::RELIEF_DEPTH),
        light,
    };
    warn_leftover("relief", &a, &["depth", "light"]);
    v
}

pub fn reflect_of(mut a: Vec<(String, SubVal)>) -> Reflect {
    let light = take_light(&mut a);
    let v = Reflect {
        gain: take_num(&mut a, "gain", defaults::REFLECT_GAIN),
        sharp: take_num(&mut a, "sharp", defaults::REFLECT_SHARP),
        light,
    };
    warn_leftover("reflect", &a, &["gain", "sharp", "light"]);
    v
}
