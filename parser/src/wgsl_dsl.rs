lalrpop_mod!(pub wgsl_dsl_grammar, "/wgsl_dsl.rs");

use wgsl_dsl_grammar::DslLineParser;
use colored::*;

/// Error from DSL parsing with position info
#[derive(Clone, Debug)]
pub struct DslError {
    pub message: String,
    pub line: usize,      // Line within the WGSL block (1-based)
    pub column: usize,    // Column (1-based)
    pub context: String,  // The problematic line
}

impl DslError {
    /// Display the error with colored output matching WGSL error style
    /// `actual_line` is the line number in the original source (already calculated by caller)
    /// `actual_column` is the column in the original source line (already calculated by caller)
    pub fn display_colored(&self, original_source: &str, actual_line: usize, actual_column: usize) {
        let start_offset = 125;
        let end_offset = 50;

        // Find the byte offset in original_source for the error line
        // Composition has structure: \nline1\nline2\nline3...
        // So newline N is followed by line N content
        let mut newline_count = 0;
        let mut line_start = 0;
        for (i, c) in original_source.char_indices() {
            if c == '\n' {
                newline_count += 1;
                if newline_count == actual_line {
                    // Line content starts after this newline
                    line_start = i + 1;
                    break;
                }
            }
        }
        let error_pos = line_start + actual_column.saturating_sub(1);

        // Calculate display window
        let feed_start = error_pos.saturating_sub(start_offset);
        let mut feed_end = (error_pos + end_offset).min(original_source.len());
        if feed_end - feed_start > 300 {
            feed_end = feed_start + 300;
        }

        // Show context with colors: cyan before error, red from error
        println!(
            "{}{}",
            &original_source[feed_start..error_pos].cyan(),
            &original_source[error_pos..feed_end].red(),
        );

        println!(
            "
            {}
            DSL error at line {}
            {}
            ",
            "working".cyan().underline(),
            actual_line.to_string().red().bold(),
            "broken".red().underline(),
        );
    }
}

/// Compile DSL syntax to WGSL code
///
/// Processes semicolon-separated statements:
/// - Statements starting with DSL commands (Xm, Ya, Seq, etc.) are parsed and compiled
/// - Other statements are passed through as raw WGSL
///
/// Multi-line constructs like Seq [...] are supported.
/// Semicolons separate statements (like WGSL).
///
/// Returns the compiled WGSL code or a DslError with position info
pub fn compile_dsl_to_wgsl(src: &str) -> Result<String, DslError> {
    let parser = DslLineParser::new();
    let mut result = Vec::new();

    // Normalize: collapse whitespace but preserve structure
    // Split by semicolons to get statements
    let statements = split_by_semicolons(src);

    for (stmt_idx, (statement, start_line)) in statements.iter().enumerate() {
        let trimmed = statement.trim();

        // Skip empty statements and comments
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") {
            result.push(trimmed.to_string());
            continue;
        }

        // Check if statement starts with a DSL command
        if starts_with_dsl_command(trimmed) {
            match parser.parse(trimmed) {
                Ok(wgsl_code) => {
                    // The parser returns a WGSL string
                    for wgsl_line in wgsl_code.lines() {
                        result.push(format!("    {}", wgsl_line));
                    }
                }
                Err(e) => {
                    let raw_column = extract_error_column(&e);
                    return Err(DslError {
                        message: format_lalrpop_error(&e, trimmed),
                        line: *start_line,
                        column: raw_column,
                        context: trimmed.to_string(),
                    });
                }
            }
        } else {
            // Pass through as raw WGSL (add semicolon back)
            result.push(format!("    {};", trimmed));
        }
    }

    Ok(result.join("\n"))
}

/// Split source by semicolons, tracking which line each statement starts on.
/// Handles multi-line statements by collapsing them.
/// Returns Vec of (statement, start_line_number)
fn split_by_semicolons(src: &str) -> Vec<(String, usize)> {
    let mut statements = Vec::new();
    let mut current_stmt = String::new();
    let mut bracket_depth: i32 = 0;
    let mut stmt_start_line = 1;
    let mut current_line = 1;

    for ch in src.chars() {
        match ch {
            '\n' => {
                current_line += 1;
                // Replace newlines with spaces to collapse multi-line statements
                if !current_stmt.trim().is_empty() {
                    current_stmt.push(' ');
                }
            }
            '[' => {
                bracket_depth += 1;
                current_stmt.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                current_stmt.push(ch);
            }
            ';' if bracket_depth == 0 => {
                // End of statement (not inside brackets)
                let trimmed = current_stmt.trim();
                if !trimmed.is_empty() {
                    statements.push((trimmed.to_string(), stmt_start_line));
                }
                current_stmt.clear();
                stmt_start_line = current_line;
            }
            _ => {
                if current_stmt.is_empty() && !ch.is_whitespace() {
                    stmt_start_line = current_line;
                }
                current_stmt.push(ch);
            }
        }
    }

    // Don't forget the last statement (if no trailing semicolon)
    let trimmed = current_stmt.trim();
    if !trimmed.is_empty() {
        statements.push((trimmed.to_string(), stmt_start_line));
    }

    statements
}

/// Extract column position from LALRPOP error
fn extract_error_column<T: std::fmt::Debug>(e: &lalrpop_util::ParseError<usize, T, &str>) -> usize {
    match e {
        lalrpop_util::ParseError::InvalidToken { location } => *location + 1,
        lalrpop_util::ParseError::UnrecognizedEof { location, .. } => *location + 1,
        lalrpop_util::ParseError::UnrecognizedToken { token: (loc, _, _), .. } => *loc + 1,
        lalrpop_util::ParseError::ExtraToken { token: (loc, _, _) } => *loc + 1,
        lalrpop_util::ParseError::User { .. } => 1,
    }
}

/// Format LALRPOP error into a human-readable message
fn format_lalrpop_error<T: std::fmt::Debug>(e: &lalrpop_util::ParseError<usize, T, &str>, input: &str) -> String {
    match e {
        lalrpop_util::ParseError::InvalidToken { location } => {
            let context = &input[*location..].chars().take(10).collect::<String>();
            format!("Invalid token at '{}...'", context)
        }
        lalrpop_util::ParseError::UnrecognizedEof { expected, .. } => {
            // Simplify expected tokens for DSL
            if expected.iter().any(|s| s.contains("[0-9]")) {
                "Unexpected end of input, expected a value (number, rational like 2/3, or variable)".to_string()
            } else {
                "Unexpected end of input".to_string()
            }
        }
        lalrpop_util::ParseError::UnrecognizedToken { token: (loc, _, _), expected, .. } => {
            // Show the actual character(s) at the error location
            let bad_char = &input[*loc..].chars().next().map(|c| c.to_string()).unwrap_or_default();
            if expected.iter().any(|s| s.contains("[0-9]")) {
                format!("Unexpected '{}', expected a value (number, rational like 2/3, or variable)", bad_char)
            } else {
                format!("Unexpected token '{}'", bad_char)
            }
        }
        lalrpop_util::ParseError::ExtraToken { token: (loc, _, _) } => {
            let bad_char = &input[*loc..].chars().next().map(|c| c.to_string()).unwrap_or_default();
            format!("Extra token '{}'", bad_char)
        }
        lalrpop_util::ParseError::User { error } => {
            format!("Parse error: {}", error)
        }
    }
}

/// Check if a line starts with a DSL command
fn starts_with_dsl_command(line: &str) -> bool {
    // Short commands need whitespace/comma check to avoid false matches
    let short_commands = ["Xm", "Xa", "Ym", "Ya", "Zm", "Za", "Sm", "Sa", "Vm", "Va", "Lm", "Am"];
    for cmd in &short_commands {
        if line.starts_with(cmd) {
            let rest = &line[cmd.len()..];
            if rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with(',') {
                return true;
            }
        }
    }
    // Longer commands - just check prefix
    if line.starts_with("Direction") || line.starts_with("Seq") || line.starts_with("Bend") || line.starts_with("Alpha") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_multiply() {
        let input = "Xm 2";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("x = x * 2"));
    }

    #[test]
    fn test_simple_add() {
        let input = "Ya 10";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("y = y + 10"));
    }

    #[test]
    fn test_float_value() {
        let input = "Zm 0.5";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("z = z * 0.5"));
    }

    #[test]
    fn test_composition() {
        let input = "Xm 2 | Ya 10";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("x = x * 2"));
        assert!(output.contains("y = y + 10"));
    }

    #[test]
    fn test_passthrough_wgsl() {
        let input = "x = x * 2.0;\ny = y + 1.0;";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("x = x * 2.0;"));
        assert!(output.contains("y = y + 1.0;"));
    }

    #[test]
    fn test_comments_preserved() {
        // Comments must be on their own statement (semicolon-separated)
        let input = "// This is a comment;\nXm 2";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// This is a comment"));
        assert!(output.contains("x = x * 2"));
    }

    #[test]
    fn test_rational() {
        let input = "Ym 2/3";
        let output = compile_dsl_to_wgsl(input).unwrap();
        // Rational 2/3 is converted to float
        assert!(output.contains("y = y * 0.666"));
    }

    #[test]
    fn test_direction() {
        let input = "Direction(0, -1, 0)";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("direction = normalize"));
        // Direction now uses runtime velocity
        assert!(output.contains("time * velocity"));
    }

    #[test]
    fn test_direction_with_velocity() {
        let input = "Direction(0, -1, 0) | Vm 2";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("direction = normalize"));
        assert!(output.contains("velocity = velocity * 2"));
    }

    #[test]
    fn test_seq() {
        let input = "Seq [ Direction(0, -1, 0) | Lm 1, Direction(1, 0, 0) | Lm 1 ]";
        let output = compile_dsl_to_wgsl(input).unwrap();
        // Seq now uses accumulating segments, not if/else
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
    }

    #[test]
    fn test_seq_multiline() {
        // Test multi-line Seq with semicolon terminator
        let input = r#"
            Seq [
                Vm 1,
                Vm 2
            ];
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
    }

    #[test]
    fn test_multiple_statements() {
        // Semicolons separate statements
        let input = "Xm 2; Ym 3";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("x = x * 2"));
        assert!(output.contains("y = y * 3"));
    }

    #[test]
    fn test_error_has_line_info() {
        let input = "Xm 2;\nYm;\nZm 3";
        let err = compile_dsl_to_wgsl(input).unwrap_err();
        assert_eq!(err.line, 2);  // Error on line 2
    }

    #[test]
    fn test_seq_with_semicolons() {
        // Seq items can be separated by semicolons (for future raw WGSL support)
        let input = "Seq [ Vm 1; Vm 2 ]";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
    }

    #[test]
    fn test_seq_multiline_with_semicolons() {
        let input = r#"
            Seq [
                Vm 1;
                Vm 2
            ]
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
    }

    #[test]
    fn test_seq_trailing_semicolon() {
        // Trailing semicolons should be allowed
        let input = r#"
            Seq [
                Vm 1;
                Vm 2;
            ]
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
    }

    #[test]
    fn test_bend_basic() {
        // Bend creates a curved path
        let input = "Direction(1, 0, 0) | Bend(0, 1, 0, 0.5)";
        let output = compile_dsl_to_wgsl(input).unwrap();
        // Should contain Bézier curve code
        assert!(output.contains("Bézier"), "Output should mention Bézier");
        assert!(output.contains("b_perp"), "Output should have perpendicular bend vector");
        assert!(output.contains("bend_raw"), "Output should have raw bend vector");
    }

    #[test]
    fn test_bend_in_seq() {
        // Bend works inside Seq
        let input = r#"
            Seq [
                Direction(1, 0, 0) | Bend(0, 1, 0, 0.5) | Lm 1;
                Direction(0, 0, -1) | Bend(1, 0, 0, -0.3) | Lm 1;
            ]
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
        assert!(output.contains("b_perp"));
    }

    #[test]
    fn test_alpha_set() {
        let input = "Alpha 0";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("alpha = 0"));
    }

    #[test]
    fn test_alpha_multiply() {
        let input = "Am 0.5";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("alpha = alpha * 0.5"));
    }

    #[test]
    fn test_alpha_in_seq() {
        let input = r#"
            Seq [
                Vm 2 | Lm 1;
                Alpha 0 | Lm 1;
            ]
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
        assert!(output.contains("alpha = 0"));
    }
}
