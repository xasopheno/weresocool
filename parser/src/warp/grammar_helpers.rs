//! Helpers for the lalrpop-generated warp grammar.
//!
//! Several warp ops accept order-independent named arguments with defaults
//! (`Bloom { threshold: 0.5, strength: 0.6 }`, `Fade { at: 30, dur: 1 }` etc.).
//! Encoding this in lalrpop directly would mean a combinatorial explosion of
//! grammar rules. Instead, the grammar parses a flat `Vec<(String, WarpExpr)>`
//! per call site and these helpers do the name lookup + default application.
//!
//! The semantics MUST match the hand-written parser in `parser.rs` exactly so
//! that every existing `cull_*.socool` composition parses identically.

use crate::warp::ast::{Chan, WarpExpr};

/// Parse a channel selector for the substance verbs (`r/g/b/a/rg/gb/rgb`).
/// Unknown → `R` with a loud warning (a typo'd channel mustn't fail silently).
pub fn parse_chan(s: &str) -> Chan {
    match s {
        "r" => Chan::R, "g" => Chan::G, "b" => Chan::B, "a" => Chan::A,
        "rg" => Chan::Rg, "gb" => Chan::Gb, "rgb" => Chan::Rgb,
        _ => {
            eprintln!("[warp] unknown channel `{}` (use r/g/b/a/rg/gb/rgb) — defaulting to r", s);
            Chan::R
        }
    }
}

/// Pull one named knob (with a default) for a single-arg substance verb, and
/// warn about any leftover keys.
pub fn take_medium_arg(mut args: Vec<(String, WarpExpr)>, op: &str, key: &str, default: f32) -> WarpExpr {
    let v = take_named_or(&mut args, key, || WarpExpr::DefaultLit(default));
    warn_leftover(op, &args, &[key]);
    v
}

/// Like `take_medium_arg` but for an op with two named scalar knobs — pull both
/// from one arg bag so leftover-checking sees them both (calling the single
/// version twice would spuriously flag the second key as unknown).
pub fn take_medium_arg2(
    mut args: Vec<(String, WarpExpr)>, op: &str,
    k1: &str, d1: f32, k2: &str, d2: f32,
) -> (WarpExpr, WarpExpr) {
    let a = take_named_or(&mut args, k1, || WarpExpr::DefaultLit(d1));
    let b = take_named_or(&mut args, k2, || WarpExpr::DefaultLit(d2));
    warn_leftover(op, &args, &[k1, k2]);
    (a, b)
}

/// Three-knob variant (`Deposit { gain, grain, cap }`).
pub fn take_medium_arg3(
    mut args: Vec<(String, WarpExpr)>, op: &str,
    k1: &str, d1: f32, k2: &str, d2: f32, k3: &str, d3: f32,
) -> (WarpExpr, WarpExpr, WarpExpr) {
    let a = take_named_or(&mut args, k1, || WarpExpr::DefaultLit(d1));
    let b = take_named_or(&mut args, k2, || WarpExpr::DefaultLit(d2));
    let c = take_named_or(&mut args, k3, || WarpExpr::DefaultLit(d3));
    warn_leftover(op, &args, &[k1, k2, k3]);
    (a, b, c)
}

/// `Background` takes a mix of scalar args (`split`, `soft`) and 3-tuple
/// args (`top: (r,g,b)`, `bottom: (r,g,b)`). The grammar tags each named
/// arg with its shape via this enum.
#[derive(Debug)]
pub enum BgArgVal {
    Scalar(WarpExpr),
    Tuple3(WarpExpr, WarpExpr, WarpExpr),
}

fn take_named(args: &mut Vec<(String, WarpExpr)>, key: &str) -> Option<WarpExpr> {
    if let Some(pos) = args.iter().position(|(n, _)| n == key) {
        Some(args.remove(pos).1)
    } else {
        None
    }
}

fn take_named_or<F: FnOnce() -> WarpExpr>(args: &mut Vec<(String, WarpExpr)>, key: &str, default: F) -> WarpExpr {
    take_named(args, key).unwrap_or_else(default)
}

/// Fold a `RATIONAL` token (`"a/b"`) to `a/b` as f32 — same arithmetic the hand
/// tokenizer used when it collapsed `INT/INT` into one rational literal.
/// Warn (loudly, once per call) about named args no extractor consumed —
/// a typo'd key (`frq:`) must not vanish silently.
fn warn_leftover(op: &str, args: &[(String, WarpExpr)], known: &[&str]) {
    for (k, _) in args {
        eprintln!(
            "[warp] {}: unknown arg `{}:` ignored (known args: {})",
            op, k, known.join(", ")
        );
    }
}

pub fn parse_rational(s: &str) -> f32 {
    let mut it = s.split('/');
    let a: f32 = it.next().unwrap().parse().unwrap();
    let b: f32 = it.next().unwrap().parse().unwrap();
    a / b
}

/// Extract two named args by name (order-independent). If both are missing,
/// returns `(Lit(0.0), Lit(0.0))` — matches the hand-parser fallback shape.
pub fn extract_named_pair(args: Vec<(String, WarpExpr)>, name_a: &str, name_b: &str) -> (WarpExpr, WarpExpr) {
    let mut args = args;
    let a = take_named_or(&mut args, name_a, || WarpExpr::DefaultLit(0.0));
    let b = take_named_or(&mut args, name_b, || WarpExpr::DefaultLit(0.0));
    (a, b)
}

/// `Osc { freq: 60.0, sync: 0.1, offset: 0.0 }`.
/// Ripple(ch, freq: 40, speed: 6, amp: 0.02) — all three tunables optional.
/// Stir(radius: 0.25, strength: 1.0) — both optional with LIVE defaults
/// (a bare `Stir bd { }` should visibly stir, not silently do nothing).
/// Displace { radial: expr, tangent: expr } — either or both; defaults 0
/// (a bare Displace is a no-op by design: the expressions ARE the op).
pub fn extract_displace_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr) {
    let radial = take_named_or(&mut args, "radial", || WarpExpr::DefaultLit(0.0));
    let tangent = take_named_or(&mut args, "tangent", || WarpExpr::DefaultLit(0.0));
    warn_leftover("Displace", &args, &["radial", "tangent"]);
    (radial, tangent)
}

pub fn extract_stir_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr) {
    let radius = take_named_or(&mut args, "radius", || WarpExpr::DefaultLit(0.25));
    let strength = take_named_or(&mut args, "strength", || WarpExpr::DefaultLit(1.0));
    warn_leftover("Stir", &args, &["radius", "strength"]);
    (radius, strength)
}

pub fn extract_ripple_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr) {
    let freq = take_named_or(&mut args, "freq", || WarpExpr::DefaultLit(40.0));
    let speed = take_named_or(&mut args, "speed", || WarpExpr::DefaultLit(6.0));
    let amp = take_named_or(&mut args, "amp", || WarpExpr::DefaultLit(0.02));
    warn_leftover("Ripple", &args, &["freq", "speed", "amp"]);
    (freq, speed, amp)

}

/// Glow(ch, r: 1, g: 0.9, b: 0.7, size: 0.25) — all tunables optional.
pub fn extract_glow_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr, WarpExpr) {
    let r = take_named_or(&mut args, "r", || WarpExpr::DefaultLit(1.0));
    let g = take_named_or(&mut args, "g", || WarpExpr::DefaultLit(0.9));
    let b = take_named_or(&mut args, "b", || WarpExpr::DefaultLit(0.7));
    let size = take_named_or(&mut args, "size", || WarpExpr::DefaultLit(0.25));
    warn_leftover("Glow", &args, &["r", "g", "b", "size"]);
    (r, g, b, size)

}

/// Relief(height: 4, lx: 0.4, ly: 0.6, ambient: 0.35) — all optional.
pub fn extract_relief_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr, WarpExpr) {
    let height = take_named_or(&mut args, "height", || WarpExpr::DefaultLit(4.0));
    let lx = take_named_or(&mut args, "lx", || WarpExpr::DefaultLit(0.4));
    let ly = take_named_or(&mut args, "ly", || WarpExpr::DefaultLit(0.6));
    let ambient = take_named_or(&mut args, "ambient", || WarpExpr::DefaultLit(0.35));
    warn_leftover("Relief", &args, &["height", "lx", "ly", "ambient"]);
    (height, lx, ly, ambient)

}

/// Stain(r: 0.93, g: 0.90, b: 0.84, strength: 2.2) — all optional.
pub fn extract_stain_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr, WarpExpr, WarpExpr) {
    let r = take_named_or(&mut args, "r", || WarpExpr::DefaultLit(0.93));
    let g = take_named_or(&mut args, "g", || WarpExpr::DefaultLit(0.90));
    let b = take_named_or(&mut args, "b", || WarpExpr::DefaultLit(0.84));
    let strength = take_named_or(&mut args, "strength", || WarpExpr::DefaultLit(2.2));
    let opacity = take_named_or(&mut args, "opacity", || WarpExpr::DefaultLit(0.35));
    warn_leftover("Stain", &args, &["r", "g", "b", "strength", "opacity"]);
    (r, g, b, strength, opacity)

}

/// Watercolor { wetness: 0.9, drying: 0.006, bleed: 2.0, deposit: 0.11, lift: 0.0 } — all optional.
pub fn extract_watercolor_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr, WarpExpr, WarpExpr) {
    // Every knob points the intuitive way (bigger = more of the word):
    // wetness = water per note, drying = per-frame water loss, bleed =
    // how hard pigment chases water, deposit = pigment per note, lift =
    // per-frame pigment loss (0 = the canvas keeps everything).
    // Old knob names get a SPECIFIC migration message — a generic
    // "unknown arg" would silently substitute defaults with inverted
    // semantics (fade 0.963 ≠ lift default 0 — a never-fading canvas).
    for (old, new_name, invert) in [
        ("wet", "wetness", false),
        ("evap", "drying", true),
        ("flow", "bleed", false),
        ("gain", "deposit", false),
        ("fade", "lift", true),
    ] {
        if let Some(i) = args.iter().position(|(k, _)| k == old) {
            args.remove(i);
            eprintln!(
                "[warp] Watercolor: `{old}:` was renamed to `{new_name}:`{} — \
                 the old value is IGNORED and the default used; update the piece",
                if invert { " (inverted: new = 1 - old)" } else { "" }
            );
        }
    }
    let wetness = take_named_or(&mut args, "wetness", || WarpExpr::DefaultLit(0.9));
    let drying = take_named_or(&mut args, "drying", || WarpExpr::DefaultLit(0.006));
    let bleed = take_named_or(&mut args, "bleed", || WarpExpr::DefaultLit(2.0));
    let deposit = take_named_or(&mut args, "deposit", || WarpExpr::DefaultLit(0.11));
    let lift = take_named_or(&mut args, "lift", || WarpExpr::DefaultLit(0.0));
    warn_leftover("Watercolor", &args, &["wetness", "drying", "bleed", "deposit", "lift"]);
    (wetness, drying, bleed, deposit, lift)
}

/// Bulge(ch, radius: 0.2, amount: 0.06, mirror: 0) — mirror=1 doubles the
/// swell at the horizontally mirrored position (for symmetric voices).
pub fn extract_bulge_args(mut args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr) {
    let radius = take_named_or(&mut args, "radius", || WarpExpr::DefaultLit(0.2));
    let amount = take_named_or(&mut args, "amount", || WarpExpr::DefaultLit(0.06));
    let mirror = take_named_or(&mut args, "mirror", || WarpExpr::DefaultLit(0.0));
    warn_leftover("Bulge", &args, &["radius", "amount", "mirror"]);
    (radius, amount, mirror)

}

pub fn extract_osc_args(args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr, WarpExpr) {
    let mut args = args;
    let freq   = take_named_or(&mut args, "freq",   || WarpExpr::DefaultLit(60.0));
    let sync   = take_named_or(&mut args, "sync",   || WarpExpr::DefaultLit(0.1));
    let offset = take_named_or(&mut args, "offset", || WarpExpr::DefaultLit(0.0));
    (freq, sync, offset)
}

/// `Noise { scale: 10.0, offset: 0.1 }`.
pub fn extract_noise_args(args: Vec<(String, WarpExpr)>) -> (WarpExpr, WarpExpr) {
    let mut args = args;
    let scale  = take_named_or(&mut args, "scale",  || WarpExpr::DefaultLit(10.0));
    let offset = take_named_or(&mut args, "offset", || WarpExpr::DefaultLit(0.1));
    (scale, offset)
}

/// `Clear(at?, every?, offset?)` — all optional, no defaults applied.
pub fn extract_clear_args(args: Vec<(String, WarpExpr)>) -> (Option<WarpExpr>, Option<WarpExpr>, Option<WarpExpr>) {
    let mut args = args;
    let at     = take_named(&mut args, "at");
    let every  = take_named(&mut args, "every");
    let offset = take_named(&mut args, "offset");
    (at, every, offset)
}

/// `Fade(at?, every?, offset?, dur: 1.0, to: 0.05)`.
pub fn extract_fade_args(args: Vec<(String, WarpExpr)>)
    -> (Option<WarpExpr>, Option<WarpExpr>, Option<WarpExpr>, WarpExpr, WarpExpr)
{
    let mut args = args;
    let at     = take_named(&mut args, "at");
    let every  = take_named(&mut args, "every");
    let offset = take_named(&mut args, "offset");
    let dur    = take_named_or(&mut args, "dur", || WarpExpr::DefaultLit(1.0));
    let to     = take_named_or(&mut args, "to",  || WarpExpr::DefaultLit(0.05));
    (at, every, offset, dur, to)
}

/// `Background { top: (r,g,b), bottom: (r,g,b), split: 0.5, soft: 0.05 }`.
/// `bot` is accepted as a short alias for `bottom` (matches hand-parser).
pub fn extract_background_args(args: Vec<(String, BgArgVal)>)
    -> (WarpExpr, WarpExpr, WarpExpr, WarpExpr, WarpExpr, WarpExpr, WarpExpr, WarpExpr)
{
    let mut top = (WarpExpr::DefaultLit(0.0), WarpExpr::DefaultLit(0.0), WarpExpr::DefaultLit(0.0));
    let mut bot = (WarpExpr::DefaultLit(0.0), WarpExpr::DefaultLit(0.0), WarpExpr::DefaultLit(0.0));
    let mut split = WarpExpr::DefaultLit(0.5);
    let mut soft  = WarpExpr::DefaultLit(0.05);
    for (name, val) in args {
        match (name.as_str(), val) {
            ("top",                BgArgVal::Tuple3(r,g,b)) => top = (r,g,b),
            ("bottom", BgArgVal::Tuple3(r,g,b)) |
            ("bot",                BgArgVal::Tuple3(r,g,b)) => bot = (r,g,b),
            ("split",              BgArgVal::Scalar(e))     => split = e,
            ("soft",               BgArgVal::Scalar(e))     => soft = e,
            _ => { /* unknown / wrong shape — silently ignored, like hand-parser */ }
        }
    }
    (top.0, top.1, top.2, bot.0, bot.1, bot.2, split, soft)
}
