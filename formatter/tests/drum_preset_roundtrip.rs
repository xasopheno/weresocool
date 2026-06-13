//! Round-trip tests for drum preset syntax. The vim plugin validates
//! formatter output by re-parsing it, so every shape must format to
//! something the parser accepts, and formatting must be idempotent.

use weresocool_formatter::{format_source, FormatConfig};

fn fmt(src: &str) -> String {
    format_source(src, &FormatConfig::default()).expect("format should succeed")
}

fn wrap(body: &str) -> String {
    format!("{{ f: 220, l: 1, g: 1, p: 0 }}\n\nmain = {{ {} }}\n", body)
}

/// format → re-parse → format must be a fixed point.
fn assert_roundtrip(body: &str) {
    let once = fmt(&wrap(body));
    let twice = fmt(&once);
    assert_eq!(once, twice, "format not idempotent for `{}`", body);
}

#[test]
fn bare_drums_roundtrip() {
    for body in ["Kick", "Snare", "HiHat", "OpenHat", "Clap", "Rimshot"] {
        let out = fmt(&wrap(body));
        assert!(out.contains(body), "output should contain `{}`:\n{}", body, out);
        assert_roundtrip(body);
    }
}

#[test]
fn preset_only_prints_without_braces() {
    let out = fmt(&wrap("Kick 808"));
    assert!(out.contains("Kick 808"), "output:\n{}", out);
    assert!(
        !out.contains("Kick 808 {"),
        "preset-only must not emit braces:\n{}",
        out
    );
    assert_roundtrip("Kick 808");
}

#[test]
fn preset_with_overrides_roundtrips() {
    let out = fmt(&wrap("Kick 808 { tune: 1/2, hump: 2 }"));
    assert!(out.contains("Kick 808 {"), "output:\n{}", out);
    assert!(out.contains("tune: 1/2"), "output:\n{}", out);
    assert_roundtrip("Kick 808 { tune: 1/2, hump: 2 }");
}

#[test]
fn named_presets_roundtrip() {
    for body in ["Snare trap", "HiHat dust", "OpenHat 909", "Snare wsc", "Clap 808", "Rimshot acoustic", "Clap trap { spread: 0.8 }"] {
        let out = fmt(&wrap(body));
        assert!(out.contains(body), "output should contain `{}`:\n{}", body, out);
        assert_roundtrip(body);
    }
}

#[test]
fn params_without_preset_still_roundtrip() {
    assert_roundtrip("Kick { hump: 2, ks_mix: 0.95 }");
    assert_roundtrip("Snare { crack: 0.85 }");
}
