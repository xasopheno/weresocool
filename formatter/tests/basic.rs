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
