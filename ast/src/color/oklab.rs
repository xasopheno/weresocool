//! OKLab — the perceptual colour space the language mixes in.
//!
//! A straight line through OKLab reads as a straight line to the eye, so the
//! midpoint between two colours is the colour you would have MIXED, and the
//! midpoint of a ramp sits halfway in LIGHTNESS. That is the whole reason it
//! is here: a gradient in sRGB bunches light or dark and reads as an uneven
//! fall of light.
//!
//! Lives in the AST crate rather than the renderer because both ends of the
//! language need it now — `color name = [mix(a, b, 0.4)]` is evaluated at
//! parse time, and the figure gradient mixes at render time. Two copies of
//! Ottosson's matrices would be two answers to "what colour is halfway".

/// sRGB (0..1, encoded) → linear.
#[inline]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear → sRGB (0..1, encoded).
#[inline]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB-encoded rgb → OKLab. Björn Ottosson's matrices.
pub fn srgb_to_oklab(c: [f32; 3]) -> [f32; 3] {
    let (r, g, b) = (
        srgb_to_linear(c[0]),
        srgb_to_linear(c[1]),
        srgb_to_linear(c[2]),
    );
    let l = 0.412_221_47 * r + 0.536_332_54 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_84 * g + 0.629_978_5 * b;
    let (l_, m_, s_) = (l.cbrt(), m.cbrt(), s.cbrt());
    [
        0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
        1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
        0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
    ]
}

/// OKLab → sRGB-encoded rgb. Clamped: a lab value can land outside the
/// display gamut and a negative channel would read as a hole in the paint.
pub fn oklab_to_srgb(lab: [f32; 3]) -> [f32; 3] {
    let l_ = lab[0] + 0.396_337_78 * lab[1] + 0.215_803_76 * lab[2];
    let m_ = lab[0] - 0.105_561_346 * lab[1] - 0.063_854_17 * lab[2];
    let s_ = lab[0] - 0.089_484_18 * lab[1] - 1.291_485_5 * lab[2];
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    let r = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
    let g = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
    let b = -0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s;
    [
        linear_to_srgb(r).clamp(0.0, 1.0),
        linear_to_srgb(g).clamp(0.0, 1.0),
        linear_to_srgb(b).clamp(0.0, 1.0),
    ]
}

/// Mix two sRGB colours perceptually. `t` 0 → `a`, 1 → `b`.
pub fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    let (la, lb) = (srgb_to_oklab(a), srgb_to_oklab(b));
    oklab_to_srgb([
        la[0] + (lb[0] - la[0]) * t,
        la[1] + (lb[1] - la[1]) * t,
        la[2] + (lb[2] - la[2]) * t,
    ])
}

/// Turn a colour's HUE by `turns` (1 = all the way round), holding its
/// lightness and its chroma.
///
/// This is what a complement is, and it is worth being explicit about because
/// the OTHER thing people write — `1 - c` — is not a complement, it is a
/// negative. It inverts lightness along with hue, so gold comes back as
/// indigo instead of as blue, and it is already the cause of one standing trap
/// in this codebase (see COLOR.md, the subtractive glaze).
pub fn rotate_hue(c: [f32; 3], turns: f32) -> [f32; 3] {
    let lab = srgb_to_oklab(c);
    let (a, b) = (lab[1], lab[2]);
    let (sin, cos) = (turns * std::f32::consts::TAU).sin_cos();
    oklab_to_srgb([lab[0], a * cos - b * sin, a * sin + b * cos])
}

/// The colour opposite this one on the hue circle. Lightness and chroma held.
pub fn complement(c: [f32; 3]) -> [f32; 3] {
    rotate_hue(c, 0.5)
}

/// Pull a colour toward neutral. `t` 0 → unchanged, 1 → grey of the same
/// lightness.
pub fn desaturate(c: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    let lab = srgb_to_oklab(c);
    oklab_to_srgb([lab[0], lab[1] * (1.0 - t), lab[2] * (1.0 - t)])
}

/// Toward black, the painter's sense of the word — `t` 0 → unchanged, 1 →
/// black.
pub fn shade(c: [f32; 3], t: f32) -> [f32; 3] {
    mix(c, [0.0, 0.0, 0.0], t)
}

/// Toward white — `t` 0 → unchanged, 1 → white.
pub fn tint(c: [f32; 3], t: f32) -> [f32; 3] {
    mix(c, [1.0, 1.0, 1.0], t)
}

/// `#rrggbb`, always six digits — the one spelling every consumer of this
/// language already parses.
pub fn to_hex(c: [f32; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[2].clamp(0.0, 1.0) * 255.0).round() as u8
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < eps)
    }

    #[test]
    fn round_trips() {
        for c in [[0.1, 0.4, 0.9], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [0.8, 0.5, 0.1]] {
            assert!(close(oklab_to_srgb(srgb_to_oklab(c)), c, 0.002), "{c:?}");
        }
    }

    #[test]
    fn a_complement_holds_lightness() {
        // The whole point, and the thing `1 - c` gets wrong: gold's opposite
        // is a BLUE of the same weight, not a dark indigo.
        let gold = [0.85, 0.65, 0.20];
        let comp = complement(gold);
        let (lg, lc) = (srgb_to_oklab(gold)[0], srgb_to_oklab(comp)[0]);
        assert!((lg - lc).abs() < 0.02, "lightness moved: {lg} vs {lc}");
        assert!(comp[2] > comp[0], "the opposite of gold is blue: {comp:?}");
        // and the naive negative does NOT hold lightness
        let naive = [1.0 - gold[0], 1.0 - gold[1], 1.0 - gold[2]];
        assert!((srgb_to_oklab(naive)[0] - lg).abs() > 0.1, "1-c should be much darker");
    }

    #[test]
    fn two_half_turns_come_home() {
        let c = [0.2, 0.6, 0.3];
        assert!(close(complement(complement(c)), c, 0.01));
    }

    #[test]
    fn desaturate_all_the_way_is_grey() {
        let g = desaturate([0.9, 0.2, 0.1], 1.0);
        assert!((g[0] - g[1]).abs() < 0.01 && (g[1] - g[2]).abs() < 0.01, "{g:?}");
    }
}
