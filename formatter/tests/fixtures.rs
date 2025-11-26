//! Fixture-based tests for the formatter
//!
//! Tests compare input files against expected output files.
//! To add a new test:
//! 1. Create `fixtures/name.socool.input`
//! 2. Create `fixtures/name.socool.expected`
//! 3. Run tests

use weresocool_formatter::{format_source, FormatConfig};

fn run_fixture_test(name: &str) {
    let input = std::fs::read_to_string(format!("tests/fixtures/{}.socool.input", name))
        .unwrap_or_else(|_| panic!("Could not read input fixture: {}", name));
    let expected = std::fs::read_to_string(format!("tests/fixtures/{}.socool.expected", name))
        .unwrap_or_else(|_| panic!("Could not read expected fixture: {}", name));

    let config = FormatConfig::default();
    let actual = format_source(&input, &config)
        .unwrap_or_else(|e| panic!("Failed to format {}: {}", name, e));

    pretty_assertions::assert_eq!(actual, expected, "Fixture test failed: {}", name);
}

#[test]
fn test_fixture_simple() {
    run_fixture_test("simple");
}

#[test]
fn test_fixture_sequence() {
    run_fixture_test("sequence");
}
