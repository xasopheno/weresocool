//! Names of the built-in drum presets.
//!
//! Only the NAMES live here — the parser validates preset tokens against
//! these lists at parse time (so typos are errors with suggestions, not
//! silent fallbacks). The actual preset VALUES live in
//! `weresocool_synth::presets`, which depends on this crate; a test there
//! asserts the two stay in sync.
//!
//! Naming philosophy: one decision gets a great sound. `wsc` is the
//! signature WereSoCool voicing and is also what a bare `Kick` / `Snare` /
//! `HiHat` resolves to.

/// The five "character" drumsets shared across every drum family: `glass`
/// (crystalline, bell-like, ringing — beautiful), `doom` (massive,
/// distorted, industrial — heavy), `tape` (warm vintage saturation,
/// rolled-off — smooth), `vapor` (dreamy, washy, detuned, long tails —
/// ambient), `neon` (bright clean synthetic electro — punchy).
const CHARACTER: &[&str] = &["glass", "doom", "tape", "vapor", "neon"];

/// Kick presets: `wsc` (signature), `808` (long sub boom), `909` (hard
/// beater click, tight), `knock` (trap: short, driven, speaker-knock),
/// `acoustic` (shell ring, low saturation), `dust` (lofi: dark, soft),
/// plus the five [`CHARACTER`] drumsets.
pub const KICK_PRESETS: &[&str] = &[
    "wsc", "808", "909", "knock", "acoustic", "dust", "glass", "doom", "tape", "vapor", "neon",
];

/// Snare presets: `wsc` (signature crack), `808` (tight dark snap),
/// `909` (tonal body + bright burst), `trap` (bright aggressive mid-crack),
/// `brush` (head-dominant, soft beater), `dust` (lofi: dark wires), plus
/// the five [`CHARACTER`] drumsets.
pub const SNARE_PRESETS: &[&str] = &[
    "wsc", "808", "909", "trap", "brush", "dust", "glass", "doom", "tape", "vapor", "neon",
];

/// HiHat presets (shared by `HiHat` and `OpenHat`): `wsc` (signature),
/// `808` (dark metallic), `909` (bright sizzle), `trap` (tight tick),
/// `acoustic` (loose, beating), `dust` (lofi: dark, airless), plus the
/// five [`CHARACTER`] drumsets.
pub const HIHAT_PRESETS: &[&str] = &[
    "wsc", "808", "909", "trap", "acoustic", "dust", "glass", "doom", "tape", "vapor", "neon",
];

/// Clap presets: `wsc` (signature), `808` (the iconic spread clap),
/// `909` (tighter, noisier), `trap` (bright layered snap), `dust`
/// (lofi: dark, papery), plus the five [`CHARACTER`] drumsets.
pub const CLAP_PRESETS: &[&str] = &[
    "wsc", "808", "909", "trap", "dust", "glass", "doom", "tape", "vapor", "neon",
];

/// Rimshot presets: `wsc` (signature), `808` (tonal ping), `909`
/// (brighter click), `acoustic` (woody side-stick), plus the five
/// [`CHARACTER`] drumsets.
pub const RIMSHOT_PRESETS: &[&str] = &[
    "wsc", "808", "909", "acoustic", "glass", "doom", "tape", "vapor", "neon",
];

/// Keep the doc comment honest: every family exposes the character set.
#[cfg(test)]
mod character_coverage {
    use super::*;
    #[test]
    fn every_family_has_the_character_drumsets() {
        for fam in [KICK_PRESETS, SNARE_PRESETS, HIHAT_PRESETS, CLAP_PRESETS, RIMSHOT_PRESETS] {
            for c in CHARACTER {
                assert!(fam.contains(c), "family missing character preset `{}`", c);
            }
        }
    }
}
