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
/// Processes each line:
/// - Lines starting with DSL commands (Xm, Ya, etc.) are parsed and compiled
/// - Other lines are passed through as raw WGSL
///
/// Returns the compiled WGSL code or a DslError with position info
pub fn compile_dsl_to_wgsl(src: &str) -> Result<String, DslError> {
    let parser = DslLineParser::new();
    let mut result = Vec::new();

    for (line_num, line) in src.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }
        if trimmed.starts_with("//") {
            result.push(line.to_string());
            continue;
        }

        // Check if line starts with a DSL command
        if starts_with_dsl_command(trimmed) {
            // Remove trailing semicolon if present for parsing
            let to_parse = trimmed.trim_end_matches(';').trim();

            // Calculate leading whitespace offset for accurate column reporting
            let leading_ws = line.len() - line.trim_start().len();

            match parser.parse(to_parse) {
                Ok(wgsl_lines) => {
                    for wgsl_line in wgsl_lines {
                        result.push(format!("    {}", wgsl_line));
                    }
                }
                Err(e) => {
                    // Extract column from LALRPOP error and add leading whitespace offset
                    let raw_column = extract_error_column(&e);
                    let column = raw_column + leading_ws;
                    return Err(DslError {
                        message: format_lalrpop_error(&e, to_parse),
                        line: line_num + 1,  // 1-based
                        column,
                        context: line.to_string(),
                    });
                }
            }
        } else {
            // Pass through as raw WGSL
            result.push(line.to_string());
        }
    }

    Ok(result.join("\n"))
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
    let commands = ["Xm", "Xa", "Ym", "Ya", "Zm", "Za", "Sm", "Sa", "Vm", "Va"];
    for cmd in &commands {
        if line.starts_with(cmd) {
            // Make sure it's followed by whitespace or end of line
            let rest = &line[cmd.len()..];
            if rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with(',') {
                return true;
            }
        }
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
        assert!(output.contains("x = x * 2.0;"));
    }

    #[test]
    fn test_simple_add() {
        let input = "Ya 10";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("y = y + 10.0;"));
    }

    #[test]
    fn test_float_value() {
        let input = "Zm 0.5";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("z = z * 0.5;"));
    }

    #[test]
    fn test_expression_value() {
        let input = "Sm (sin(time) * 0.5 + 1.0)";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("scale = scale * (sin(time) * 0.5 + 1.0);"));
    }

    #[test]
    fn test_comma_separated() {
        let input = "Xm 2, Ya 10, Zm 0.5";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("x = x * 2.0;"));
        assert!(output.contains("y = y + 10.0;"));
        assert!(output.contains("z = z * 0.5;"));
    }

    #[test]
    fn test_mixed_dsl_and_wgsl() {
        let input = "Xm 2\nlet temp = x * 2.0;\nYa temp";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("x = x * 2.0;"));
        assert!(output.contains("let temp = x * 2.0;"));
        assert!(output.contains("y = y + temp;"));
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
        let input = "// This is a comment\nXm 2";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("// This is a comment"));
        assert!(output.contains("x = x * 2.0;"));
    }

    #[test]
    fn test_variable_value() {
        let input = "Ya time";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("y = y + time;"));
    }

    #[test]
    fn test_rational() {
        let input = "Ym 2/3";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("y = y * 2.0 / 3.0;"));
    }

    #[test]
    fn test_error_has_line_info() {
        let input = "Xm 2\nYm\nZm 3";
        let err = compile_dsl_to_wgsl(input).unwrap_err();
        assert_eq!(err.line, 2);  // Error on line 2
    }
}
