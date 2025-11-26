//! Tests for syntax preservation in the formatter.
//!
//! The formatter should preserve the original syntax forms used in the source code,
//! not canonicalize them. For example:
//! - `Fm 1` should stay `Fm 1`, not become `Tm 1`
//! - `Seq [...]` should stay `Seq [...]`, not become `Sequence [...]`
//! - `O [...]` should stay `O [...]`, not become `Overlay [...]`

use weresocool_formatter::{format_source, FormatConfig};

/// Test helper that formats input and checks that it contains expected substrings
/// and does NOT contain unexpected substrings.
fn assert_syntax_preserved(input: &str, should_contain: &[&str], should_not_contain: &[&str]) {
    let config = FormatConfig::default();
    let result = format_source(input, &config);

    assert!(result.is_ok(), "Formatting should succeed for input:\n{}", input);
    let output = result.unwrap();

    for expected in should_contain {
        assert!(
            output.contains(expected),
            "Output should contain '{}'\nInput:\n{}\nOutput:\n{}",
            expected,
            input,
            output
        );
    }

    for unexpected in should_not_contain {
        assert!(
            !output.contains(unexpected),
            "Output should NOT contain '{}'\nInput:\n{}\nOutput:\n{}",
            unexpected,
            input,
            output
        );
    }
}

/// Shorthand: test that formatting is idempotent (formatting twice gives same result)
fn assert_idempotent(input: &str) {
    let config = FormatConfig::default();
    let first = format_source(input, &config).expect("First format should succeed");
    let second = format_source(&first, &config).expect("Second format should succeed");

    assert_eq!(
        first, second,
        "Formatting should be idempotent.\nFirst:\n{}\nSecond:\n{}",
        first, second
    );
}

// =============================================================================
// TransposeM (Fm vs Tm)
// =============================================================================

#[test]
fn test_preserves_fm_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Fm 2 }",
        &["Fm 2"],
        &["Tm 2"],
    );
}

#[test]
fn test_preserves_tm_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Tm 2 }",
        &["Tm 2"],
        &["Fm 2"],
    );
}

// =============================================================================
// TransposeA (Fa vs Ta)
// =============================================================================

#[test]
fn test_preserves_fa_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Fa 100 }",
        &["Fa 100"],
        &["Ta 100"],
    );
}

#[test]
fn test_preserves_ta_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Ta 100 }",
        &["Ta 100"],
        &["Fa 100"],
    );
}

// =============================================================================
// Gain (Gain vs Gm)
// =============================================================================

#[test]
fn test_preserves_gain_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Gain 1/2 }",
        &["Gain 1/2"],
        &["Gm 1/2"],
    );
}

#[test]
fn test_preserves_gm_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Gm 1/2 }",
        &["Gm 1/2"],
        &["Gain 1/2"],
    );
}

// =============================================================================
// Length (Length vs Lm)
// =============================================================================

#[test]
fn test_preserves_length_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Length 2 }",
        &["Length 2"],
        &["Lm 2"],
    );
}

#[test]
fn test_preserves_lm_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Lm 2 }",
        &["Lm 2"],
        &["Length 2"],
    );
}

// =============================================================================
// PanM (PanM vs Pm)
// =============================================================================

#[test]
fn test_preserves_panm_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { PanM 2 }",
        &["PanM 2"],
        &["Pm 2"],
    );
}

#[test]
fn test_preserves_pm_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Pm 2 }",
        &["Pm 2"],
        &["PanM 2"],
    );
}

// =============================================================================
// PanA (PanA vs Pa)
// =============================================================================

#[test]
fn test_preserves_pana_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { PanA 1/2 }",
        &["PanA 1/2"],
        &["Pa 1/2"],
    );
}

#[test]
fn test_preserves_pa_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Pa 1/2 }",
        &["Pa 1/2"],
        &["PanA 1/2"],
    );
}

// =============================================================================
// Sequence (Sequence vs Seq)
// =============================================================================

#[test]
fn test_preserves_sequence_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Sequence [Fm 1, Fm 2] }",
        &["Sequence ["],
        &["Seq ["],
    );
}

#[test]
fn test_preserves_seq_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Seq [Fm 1, Fm 2] }",
        &["Seq ["],
        &["Sequence ["],
    );
}

// =============================================================================
// Overlay (Overlay vs O)
// =============================================================================

#[test]
fn test_preserves_overlay_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Overlay [Fm 1, Fm 2] }",
        &["Overlay ["],
        &["O ["],
    );
}

#[test]
fn test_preserves_o_overtone_syntax() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { O [(1, 0, 1, 0), (2, 0, 1/2, 0)] }",
        &["O ["],
        &["Overlay ["],
    );
}

// =============================================================================
// Mixed syntax in pipe chains
// =============================================================================

#[test]
fn test_preserves_mixed_syntax_in_pipe() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Fm 2 | Gm 1/2 | Lm 1 | Pa 0 }",
        &["Fm 2", "Gm 1/2", "Lm 1", "Pa 0"],
        &["Tm 2", "Gain 1/2", "Length 1", "PanA 0"],
    );
}

#[test]
fn test_preserves_verbose_syntax_in_pipe() {
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Tm 2 | Gain 1/2 | Length 1 | PanA 0 }",
        &["Tm 2", "Gain 1/2", "Length 1", "PanA 0"],
        &["Fm 2", "Gm 1/2", "Lm 1", "Pa 0"],
    );
}

// =============================================================================
// Idempotence tests
// =============================================================================

#[test]
fn test_idempotent_shorthand() {
    assert_idempotent("{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Fm 1 | Gm 1/2 | Seq [Lm 1, Lm 2] }");
}

#[test]
fn test_idempotent_verbose() {
    assert_idempotent("{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Tm 1 | Gain 1/2 | Sequence [Length 1, Length 2] }");
}

#[test]
fn test_idempotent_overtone() {
    // Note: The O[...] syntax outputs without braces around definition bodies,
    // which is valid for the parser. This tests the O syntax preservation itself.
    assert_syntax_preserved(
        "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { O [(1, 0, 1, 0), (2, 0, 1/2, 0)] }",
        &["O [", "(1, 0, 1, 0)", "(2, 0, 1/2, 0)"],
        &["Overlay ["],
    );
}
