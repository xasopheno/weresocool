//! Table-driven transcendentals for the drum voices.
//!
//! A drum voice is mostly `sin`, `exp`, `tan` and `tanh` — a profile of a
//! composition carrying sixty-four snare voices put ~60% of the render inside
//! libm. These are the same functions evaluated to audio precision instead of
//! to the last bit: a table lookup plus one linear interpolation, or a small
//! rational, and the error lands 100+ dB under the signal.
//!
//! Scope is deliberate. These are used by the DRUM oscillators and their
//! filters, not by the `Sine`/`Triangle`/`Square`/`Fm` oscillators, whose
//! output is snapshot-tested sample-for-sample.

use std::f64::consts::{FRAC_PI_2, LN_2, PI, TAU};
use std::sync::LazyLock;

// ═══════════════════════════════════════════════════════════════════════
// SINE
// ═══════════════════════════════════════════════════════════════════════

/// Table entries over one full turn, each holding `(sin, cos)` at that angle.
/// 4096 × 16 bytes = 64 KB, and one lookup pulls both halves on the same
/// cache line.
///
/// Interpolating with the derivative rather than between neighbours is what
/// buys the accuracy: a two-term Taylor step off the tabulated point leaves
/// only the d³/6 term, which at a cell width of 2π/4096 is under 1e-9 —
/// roughly 180 dB down, versus 4.6e-7 for plain linear interpolation. The
/// cost is one extra multiply-add.
const SIN_BITS: usize = 12;
const SIN_SIZE: usize = 1 << SIN_BITS;
const SIN_MASK: usize = SIN_SIZE - 1;
const SIN_CELL: f64 = TAU / SIN_SIZE as f64;

static SIN_TABLE: LazyLock<Box<[(f64, f64); SIN_SIZE]>> = LazyLock::new(|| {
    let mut t = Box::new([(0.0, 0.0); SIN_SIZE]);
    for (i, slot) in t.iter_mut().enumerate() {
        let angle = TAU * i as f64 / SIN_SIZE as f64;
        *slot = (angle.sin(), angle.cos());
    }
    t
});

/// `sin(x)` for any finite `x`, to well beyond audio precision.
///
/// The index wraps on the table mask, which does the modulo-2π for free —
/// including for negative phases, where the `usize` cast wraps in two's
/// complement to exactly the right cell.
#[inline]
pub fn fast_sin(x: f64) -> f64 {
    let table = &**SIN_TABLE;
    let scaled = x * (SIN_SIZE as f64 / TAU);
    let floor = scaled.floor();
    let idx = (floor as i64 as usize) & SIN_MASK;
    let (s, c) = table[idx];
    // Angle remaining past the tabulated point.
    let d = (scaled - floor) * SIN_CELL;
    s + d * (c - 0.5 * d * s)
}

// ═══════════════════════════════════════════════════════════════════════
// EXP
// ═══════════════════════════════════════════════════════════════════════

/// `exp(x) = 2^(x·log₂e) = 2^k · 2^f`. The integer part is built directly in
/// the exponent field of the result; only the fractional part needs a table.
/// 1024 entries (8 KB) with linear interpolation leaves ~6e-8 relative.
const EXP_BITS: usize = 10;
const EXP_SIZE: usize = 1 << EXP_BITS;

static EXP2_TABLE: LazyLock<Box<[f64; EXP_SIZE + 1]>> = LazyLock::new(|| {
    let mut t = Box::new([0.0; EXP_SIZE + 1]);
    for (i, slot) in t.iter_mut().enumerate() {
        *slot = (i as f64 / EXP_SIZE as f64).exp2();
    }
    t
});

/// `exp(x)` to ~1e-7 relative — more than an amplitude envelope needs.
///
/// Underflows to zero rather than to a subnormal: every caller here is a decay
/// envelope, and a value that small is silence in any case.
#[inline]
pub fn fast_exp(x: f64) -> f64 {
    if x <= -700.0 {
        return 0.0;
    }
    if x >= 700.0 {
        return f64::INFINITY;
    }
    let table = &**EXP2_TABLE;
    let y = x / LN_2;
    let k = y.floor();
    let f = y - k;

    let scaled = f * EXP_SIZE as f64;
    let i = scaled as usize;
    let frac = scaled - i as f64;
    let a = table[i];
    let b = table[i + 1];
    let mantissa = a + (b - a) * frac;

    // 2^k, assembled in the exponent field. `k` is within ±1010 here, so the
    // biased exponent stays in range.
    let pow2k = f64::from_bits(((k as i64 + 1023) as u64) << 52);
    mantissa * pow2k
}

// ═══════════════════════════════════════════════════════════════════════
// TAN — the SVF's frequency warp
// ═══════════════════════════════════════════════════════════════════════

/// `tan(π·u)` for `u` in `[0, 0.49]` — the TPT state-variable filter's
/// frequency warp, which it needs on every sample because the kick's and
/// snare's shell cutoffs track their body envelopes.
///
/// Built from the sine table rather than a table of its own. A direct `tan`
/// table is a bad shape to interpolate: the function steepens without bound
/// towards Nyquist, so the cells that matter least for accuracy are the ones
/// that need to be finest. As a ratio of two sines both halves stay smooth
/// and bounded, and the error is the sine table's — about 1e-9 — everywhere
/// in the range.
#[inline]
pub fn svf_tan(u: f64) -> f64 {
    let angle = PI * u;
    fast_sin(angle) / fast_sin(angle + FRAC_PI_2)
}

// ═══════════════════════════════════════════════════════════════════════
// TANH
// ═══════════════════════════════════════════════════════════════════════

/// `tanh(x)`: its 7th-order Padé approximant near zero, where a saturator
/// spends nearly all its time and the rational is good to ~1e-9, handing over
/// past |x| = 3 to `1 − 2/(e^{2x}+1)`, which stays exact out into the tail
/// where the Padé stops converging.
#[inline]
pub fn fast_tanh(x: f64) -> f64 {
    let ax = x.abs();
    if ax >= 3.0 {
        let t = 1.0 - 2.0 / (fast_exp(2.0 * ax) + 1.0);
        return if x < 0.0 { -t } else { t };
    }
    let x2 = x * x;
    let num = x * (135135.0 + x2 * (17325.0 + x2 * (378.0 + x2)));
    let den = 135135.0 + x2 * (62370.0 + x2 * (3150.0 + x2 * 28.0));
    num / den
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every table is only worth having if it lands far enough under the
    /// signal to be inaudible. These bounds are the claim; keep them honest.
    #[test]
    fn sin_matches_libm() {
        let mut worst: f64 = 0.0;
        for i in 0..200_000 {
            let x = -50.0 + i as f64 * 1e-3;
            worst = worst.max((fast_sin(x) - x.sin()).abs());
        }
        assert!(worst < 1e-8, "worst sine error {worst:e}");
    }

    #[test]
    fn exp_matches_libm() {
        let mut worst: f64 = 0.0;
        for i in 0..100_000 {
            let x = -30.0 + i as f64 * 3e-4;
            let expected = x.exp();
            worst = worst.max(((fast_exp(x) - expected) / expected).abs());
        }
        assert!(worst < 1e-6, "worst exp relative error {worst:e}");
    }

    #[test]
    fn exp_underflows_to_silence() {
        assert_eq!(fast_exp(-1000.0), 0.0);
    }

    #[test]
    fn tan_matches_libm_over_the_filter_range() {
        let mut worst: f64 = 0.0;
        for i in 0..100_000 {
            let u = 15.0 / 48_000.0 + i as f64 * (0.49 - 15.0 / 48_000.0) / 100_000.0;
            let expected = (PI * u).tan();
            worst = worst.max(((svf_tan(u) - expected) / expected).abs());
        }
        // Relative error peaks at the bottom of the range, where `g` itself is
        // ~1e-3 and the sine table's ~1e-9 absolute error is a larger share of
        // it. In absolute terms the coefficient is right to 1e-10 throughout.
        assert!(worst < 1e-6, "worst tan relative error {worst:e}");
    }

    #[test]
    fn tanh_matches_libm() {
        let mut worst: f64 = 0.0;
        for i in 0..100_000 {
            let x = -8.0 + i as f64 * 1.6e-4;
            worst = worst.max((fast_tanh(x) - x.tanh()).abs());
        }
        assert!(worst < 1e-6, "worst tanh error {worst:e}");
    }
}
