use weresocool_formatter::{format_source, FormatConfig};

#[test]
fn test_format_simple() {
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Overlay [
    Fm 1,
    Fm 2
  ]
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config);

    assert!(result.is_ok(), "Formatting should succeed");
    let output = result.unwrap();

    // Should contain init block
    assert!(output.contains("f: 220"), "Should contain f: 220");
    assert!(output.contains("l: 1"), "Should contain l: 1");

    // Should contain main definition
    assert!(output.contains("main ="), "Should contain main definition");
}

#[test]
fn test_format_init_block() {
    let input = "{ f: 311, l: 1, g: 1/2, p: 0 }\n\nmain = { Fm 1 }";

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // Check init block is formatted correctly
    assert!(result.contains("{ f: 311, l: 1, g: 1/2, p: 0 }"));
}

#[test]
fn test_format_sequence() {
    let input = "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Seq [Fm 1, Fm 2, Fm 3] }";

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    assert!(result.contains("Seq"));
    assert!(result.contains("Fm 1")); // TransposeM is formatted as Fm
}

#[test]
fn test_format_overlay() {
    let input = "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Overlay [Fm 1, Fm 2] }";

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    assert!(result.contains("Overlay"));
}

#[test]
fn test_format_pipe_chain() {
    let input = "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Fm 1 | Gain 1/2 | Length 2 }";

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    assert!(result.contains("Fm 1"));
    assert!(result.contains("Gain 1/2"));
    assert!(result.contains("Length 2"));
}

#[test]
fn test_preserves_float_values() {
    // The key issue: floats like 9.0 should stay as 9.0, not become 9
    let input = "{ f: 311.127, l: 1, g: 1, p: 0 }\n\nthing = { Fm 9.0 | Lm 1/2 }";

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // Float 9.0 should be preserved, not converted to 9
    assert!(result.contains("9.0"), "Float 9.0 should be preserved. Got:\n{}", result);
}

#[test]
fn test_preserves_keyword_variants() {
    // Tm should stay as Tm, not become Fm
    let input = "{ f: 220, l: 1, g: 1, p: 0 }\n\nmain = { Tm 2 | Lm 1/2 }";

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // Tm should be preserved
    assert!(result.contains("Tm 2"), "Tm keyword should be preserved. Got:\n{}", result);
}

#[test]
fn test_preserves_comments_before_definition() {
    let input = r#"-- This is a comment
{ f: 220, l: 1, g: 1, p: 0 }

-- Define main
main = { Fm 1 }"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    assert!(result.contains("-- This is a comment"), "Top comment should be preserved. Got:\n{}", result);
    assert!(result.contains("-- Define main"), "Definition comment should be preserved. Got:\n{}", result);
}

#[test]
fn test_preserves_comments_inside_overlay() {
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Overlay [
    Fm 1,
    -- This is commented out
    Fm 2,
  ]
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    assert!(result.contains("-- This is commented out"), "Comment inside Overlay should be preserved. Got:\n{}", result);
}

#[test]
fn test_comment_indentation_matches_context() {
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Overlay [
    Fm 1,
    -- comment here
    Fm 2,
  ]
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // The comment should have the same indentation as "Fm 2," (4 spaces)
    assert!(result.contains("    -- comment here"),
        "Comment should have 4-space indentation to match Overlay contents. Got:\n{}", result);
}

#[test]
fn test_no_extra_blank_lines_around_comments() {
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Overlay [
    Fm 1,
    -- comment 1
    -- comment 2
    Fm 2,
  ]
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // Should not have double blank lines around comments
    assert!(!result.contains("\n\n\n"), "Should not have triple newlines. Got:\n{}", result);

    // Comments should be consecutive without blank lines between them
    assert!(result.contains("-- comment 1\n    -- comment 2"),
        "Consecutive comments should not have blank lines between them. Got:\n{}", result);
}

#[test]
fn test_preserves_wgsl_content() {
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Fm 1 | wgsl {
    x = x + sin(time);
    y = y * 2.0;
  }
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    assert!(result.contains("x = x + sin(time);"), "WGSL content should be preserved. Got:\n{}", result);
    assert!(result.contains("y = y * 2.0;"), "WGSL content should be preserved. Got:\n{}", result);
}

#[test]
fn test_comment_before_closing_bracket_indentation() {
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Overlay [
    Fm 1,
    Fm 2,
    -- trailing comment
  ]
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // Comment before ] should have same indent as content (4 spaces), not bracket indent (2 spaces)
    assert!(result.contains("    -- trailing comment"),
        "Comment before closing bracket should have 4-space indentation. Got:\n{}", result);
}

#[test]
fn test_collapses_single_line_bracket_content() {
    // When content and ] are on the same line after [, collapse to one line
    let input = r#"{ f: 220, l: 1, g: 1, p: 0 }

main = {
  Fm 1 | Overlay [
    Fm 1, Fm 2]
}"#;

    let config = FormatConfig::default();
    let result = format_source(input, &config).unwrap();

    // Should be collapsed to one line
    assert!(result.contains("Overlay [Fm 1, Fm 2]"),
        "Single-line bracket content should be collapsed. Got:\n{}", result);
}
