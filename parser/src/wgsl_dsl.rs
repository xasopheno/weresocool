lalrpop_mod!(pub wgsl_dsl_grammar, "/wgsl_dsl.rs");

use wgsl_dsl_grammar::DslLineParser;
use weresocool_error::ErrorDisplay;

/// Error from DSL parsing with position info
#[derive(Clone, Debug)]
pub struct DslError {
    pub message: String,
    pub line: usize,      // Line within the WGSL block (1-based)
    pub column: usize,    // Column (1-based)
    pub context: String,  // The problematic line
}

impl DslError {
    /// Display the error with colored output
    /// `actual_line` is the line number in the original source (already calculated by caller)
    /// `actual_column` is the column in the original source line (already calculated by caller)
    pub fn display_colored(&self, original_source: &str, actual_line: usize, actual_column: usize, quiet: bool) {
        ErrorDisplay {
            source: original_source,
            line: actual_line,
            column: actual_column,
            label: "DSL error",
            use_cyan: true,
            ..Default::default()
        }.display(quiet);
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
        // Convert -- style comments to // for WGSL compatibility
        if trimmed.starts_with("--") {
            result.push(format!("    //{}", &trimmed[2..]));
            continue;
        }
        // Preserve // style comments
        if trimmed.starts_with("//") {
            result.push(format!("    {}", trimmed));
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
            // Color-law guard: a color channel must never be set to an
            // absolute constant in a wgsl block. Absolute sets discard the
            // brush palette + draw tint and diverge between the renderer's two
            // color-composition paths (the DSL color ops Bm/Ba/Die are all
            // relative for exactly this reason). We reject only the
            // unambiguous footgun — `red = 0.5` — and leave expressions alone.
            if let Some(chan) = absolute_color_literal_channel(trimmed) {
                return Err(DslError {
                    message: format!(
                        "color channel `{chan}` set to an absolute constant — color ops must be \
                         relative (e.g. `{chan} = {chan} * 0.5`, or use `Bm`/`Ba`). An absolute \
                         set wipes the brush palette and tint."
                    ),
                    line: *start_line,
                    column: 1,
                    context: trimmed.to_string(),
                });
            }
            // Pass through as raw WGSL (add semicolon back)
            result.push(format!("    {};", trimmed));
        }
    }

    Ok(rewrite_surface_idents(&result.join("\n")))
}

/// Rewrite the Law-4 surface identifiers to the raw WGSL uniform/instance
/// field names naga actually sees. A brush `wgsl { … }` block is raw WGSL
/// underneath, so the dotted note fields (`note.t`, `note.l`, `note.gain`)
/// and the shared live-clock name (`clock`) can't reach the shader verbatim —
/// this boundary maps them onto the instance struct's flat fields. The old
/// underscore spellings already match those fields, so they pass through
/// untouched and keep working. Word-bounded so `account`/`clockwise`/etc. are
/// safe; the dotted forms are matched literally.
fn rewrite_surface_idents(wgsl: &str) -> String {
    use regex::Regex;
    let mut out = wgsl.to_string();
    // Dotted note fields → flat instance-struct fields.
    for (from, to) in [
        (r"\bnote\.t\b", "note_t"),
        (r"\bnote\.l\b", "note_l"),
        (r"\bnote\.gain\b", "note_gain"),
        (r"\bnote\.event\b", "note_event"),
    ] {
        out = Regex::new(from).unwrap().replace_all(&out, to).into_owned();
    }
    // Bare atoms: `count` is the note-index sugar; `clock` is the live play
    // clock (the `song_time` uniform in the brush shader).
    out = Regex::new(r"\bcount\b").unwrap().replace_all(&out, "note_event").into_owned();
    out = Regex::new(r"\bclock\b").unwrap().replace_all(&out, "song_time").into_owned();
    out
}

/// If `stmt` is `red|green|blue|alpha = <numeric literal>` (an absolute color
/// set), return the channel name. Returns `None` for any expression RHS — only
/// the unambiguous constant case is rejected, so relative forms like
/// `red = red * f` or `red = some_var` still pass.
fn absolute_color_literal_channel(stmt: &str) -> Option<&'static str> {
    let (lhs, rhs) = stmt.split_once('=')?;
    // Guard against `==`, `<=`, etc. — only a plain assignment.
    if rhs.starts_with('=') || lhs.ends_with(['<', '>', '!']) {
        return None;
    }
    // Only the palette channels (rgb). `alpha` is derived downstream from
    // max(r,g,b), not a palette layer, so an absolute alpha set isn't the
    // palette-stomp this guards (and shipped comps set it directly).
    let channel = match lhs.trim() {
        "red" => "red",
        "green" => "green",
        "blue" => "blue",
        _ => return None,
    };
    // Absolute iff the RHS is purely a numeric literal (no identifiers/ops that
    // could make it relative). `0.5`, `1`, `-0.2`, `.3` → absolute.
    let rhs = rhs.trim().trim_end_matches(';').trim();
    let is_number = !rhs.is_empty()
        && rhs.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+')
        && rhs.chars().any(|c| c.is_ascii_digit());
    if is_number { Some(channel) } else { None }
}

/// Split source by semicolons, tracking which line each statement starts on.
/// Handles multi-line statements by collapsing them.
/// Preserves comment lines (-- or //) as separate statements at top level.
/// Skips comment lines inside brackets (they're preserved in the original source).
/// Returns Vec of (statement, start_line_number)
fn split_by_semicolons(src: &str) -> Vec<(String, usize)> {
    let mut statements = Vec::new();
    let mut current_stmt = String::new();
    let mut bracket_depth: i32 = 0;
    // Start at 0 so line numbers are offsets from block start (added to block_start_line in caller)
    let mut stmt_start_line = 0;
    let mut current_line = 0;

    for line in src.lines() {
        let trimmed_line = line.trim();

        // Check if this line is a comment
        let is_comment = trimmed_line.starts_with("--") || trimmed_line.starts_with("//");

        if is_comment {
            if bracket_depth == 0 {
                // Top-level comment: emit any pending statement, then emit comment
                let trimmed = current_stmt.trim();
                if !trimmed.is_empty() {
                    statements.push((trimmed.to_string(), stmt_start_line));
                    current_stmt.clear();
                }
                // Emit the comment as its own statement
                statements.push((trimmed_line.to_string(), current_line));
                stmt_start_line = current_line + 1;
            }
            // Comments inside brackets are skipped (they're in the original source for formatting)
            current_line += 1;
            continue;
        }

        // Strip trailing comments from the line (-- or //)
        let line_without_comment = if let Some(pos) = line.find("--") {
            &line[..pos]
        } else if let Some(pos) = line.find("//") {
            &line[..pos]
        } else {
            line
        };

        // Process the line character by character
        for ch in line_without_comment.chars() {
            match ch {
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

        // Add space between lines (for multi-line statements)
        if !current_stmt.trim().is_empty() {
            current_stmt.push(' ');
        }
        current_line += 1;
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

/// Simplify LALRPOP expected token names to human-readable form
fn simplify_expected_tokens(expected: &[String]) -> String {
    let simplified: Vec<&str> = expected.iter()
        .filter_map(|s| {
            match s.as_str() {
                r#""|""# => Some("|"),
                r#"",""# => Some(","),
                r#"";""# => Some(";"),
                r#""]""# => Some("]"),
                r#""[""# => Some("["),
                r#""(""# => Some("("),
                r#"")""# => Some(")"),
                r#""/""# => Some("/"),
                r#""Seq""# => Some("Seq"),
                r#""Direction""# => Some("Direction"),
                r#""Bend""# => Some("Bend"),
                r#""ArcTo""# => Some("ArcTo"),
                r#""Alpha""# => Some("Alpha"),
                r#""Xm""# => Some("Xm"),
                r#""Xa""# => Some("Xa"),
                r#""Ym""# => Some("Ym"),
                r#""Ya""# => Some("Ya"),
                r#""Zm""# => Some("Zm"),
                r#""Za""# => Some("Za"),
                r#""Sm""# => Some("Sm"),
                r#""Sa""# => Some("Sa"),
                r#""Vm""# => Some("Vm"),
                r#""Va""# => Some("Va"),
                r#""Lm""# => Some("Lm"),
                r#""Am""# => Some("Am"),
                s if s.contains("[0-9]") => Some("<number>"),
                _ => None
            }
        })
        .collect();

    if simplified.is_empty() {
        "valid token".to_string()
    } else {
        simplified.join(", ")
    }
}

/// Format LALRPOP error into a human-readable message
fn format_lalrpop_error<T: std::fmt::Debug>(e: &lalrpop_util::ParseError<usize, T, &str>, input: &str) -> String {
    match e {
        lalrpop_util::ParseError::InvalidToken { location } => {
            let context = &input[*location..].chars().take(20).collect::<String>();
            format!("Invalid token at '{}...'", context)
        }
        lalrpop_util::ParseError::UnrecognizedEof { expected, .. } => {
            let expected_str = simplify_expected_tokens(expected);
            format!("Unexpected end of input\nExpected: {}", expected_str)
        }
        lalrpop_util::ParseError::UnrecognizedToken { token: (loc, _, _), expected, .. } => {
            // Show the actual token at the error location (first word or character)
            let bad_token: String = input[*loc..].chars()
                .take_while(|c| !c.is_whitespace() && *c != ',' && *c != ';' && *c != ')' && *c != ']')
                .take(20)
                .collect();
            let expected_str = simplify_expected_tokens(expected);
            format!("Unexpected '{}'\nExpected: {}", bad_token, expected_str)
        }
        lalrpop_util::ParseError::ExtraToken { token: (loc, _, _) } => {
            let bad_token: String = input[*loc..].chars()
                .take_while(|c| !c.is_whitespace())
                .take(20)
                .collect();
            format!("Extra token '{}' after valid expression", bad_token)
        }
        lalrpop_util::ParseError::User { error } => {
            format!("Parse error: {}", error)
        }
    }
}

/// Check if a line starts with a DSL command
fn starts_with_dsl_command(line: &str) -> bool {
    // Short commands need whitespace/comma check to avoid false matches
    let short_commands = ["Xm", "Xa", "Ym", "Ya", "Zm", "Za", "Smx", "Smy", "Smz", "Sm", "Sa", "Vm", "Va", "Lm", "Am", "Bm", "Ba", "Die", "Rx", "Ry", "Rz", "AsIs", "None"];
    for cmd in &short_commands {
        if line.starts_with(cmd) {
            let rest = &line[cmd.len()..];
            if rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with(',') {
                return true;
            }
        }
    }
    // Longer commands - just check prefix
    if line.starts_with("Direction") || line.starts_with("Seq") || line.starts_with("Bend") || line.starts_with("ArcTo") || line.starts_with("Alpha") {
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
    fn test_die_lifetime() {
        let input = "Die 4";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("_die_end = (4"), "death time baked in: {output}");
        assert!(output.contains("scale = 0.0"), "scale snaps to 0: {output}");
        assert!(output.contains("smoothstep"), "fade before death: {output}");
    }

    #[test]
    fn test_die_composes_with_pipe() {
        let input = "Sm 0.3 | Die 2";
        let output = compile_dsl_to_wgsl(input).unwrap();
        assert!(output.contains("scale = scale * 0.3"), "{output}");
        assert!(output.contains("_die_end = (2"), "{output}");
    }

    #[test]
    fn note_dot_fields_rewrite_to_instance_fields() {
        // Law-4 surface: `note.t/l/gain` and `clock` are the beautiful spelling;
        // they lower to the flat instance-struct fields naga sees.
        // Quoted expression:
        let o = compile_dsl_to_wgsl(r#"Sm "0.9 + note.gain * 0.9""#).unwrap();
        assert!(o.contains("note_gain"), "quoted note.gain -> note_gain: {o}");
        assert!(!o.contains("note.gain"), "dotted form must not survive: {o}");
        // Bare leading dotted identifier (the grammar-extension case):
        let o = compile_dsl_to_wgsl("Bm note.gain * 1.0 + 0.3").unwrap();
        assert!(o.contains("note_gain"), "bare note.gain -> note_gain: {o}");
        // Simple bare dotted value + the other fields + clock:
        let o = compile_dsl_to_wgsl("Sm note.t | Vm note.l | Xa clock").unwrap();
        assert!(o.contains("note_t") && o.contains("note_l") && o.contains("song_time"),
            "note.t/note.l/clock all rewrite: {o}");
        // `count` sugar and passthrough RawWgsl RHS both get rewritten:
        let o = compile_dsl_to_wgsl("red = red * note.gain;").unwrap();
        assert!(o.contains("note_gain"), "raw-wgsl RHS note.gain -> note_gain: {o}");
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
    fn test_double_dash_comments_converted() {
        // -- style comments should be converted to // for WGSL compatibility
        let input = "Xm 2;\n-- this is a comment\nYm 3";
        let output = compile_dsl_to_wgsl(input).unwrap();
        println!("Output:\n{}", output);
        assert!(output.contains("// this is a comment"), "-- comment should be converted to //");
        assert!(!output.contains("--"), "-- should not appear in WGSL output");
        assert!(output.contains("x = x * 2"));
        assert!(output.contains("y = y * 3"));
    }

    #[test]
    fn test_trailing_comment_stripped() {
        // Trailing -- comments should be stripped from code lines
        let input = r#"
            Seq [
                Direction(1, 0, 0) | Lm 1;
                Direction(0, 1, 0) | Lm 1;
            ] -- this is a trailing comment
        "#;
        let result = compile_dsl_to_wgsl(input);
        assert!(result.is_ok(), "Trailing comment should not cause parse error: {:?}", result.err());
        let output = result.unwrap();
        println!("Output:\n{}", output);
        // Should have 2 segments
        assert!(output.contains("// Segment 0"));
        assert!(output.contains("// Segment 1"));
    }

    #[test]
    fn test_comment_in_seq() {
        // Comments inside Seq are skipped in compiled output (they're preserved in original source)
        // The formatter uses the original source, so comments are preserved for display
        let input = r#"
            Seq [
                Direction(1, 0, 0) | Lm 8;
                -- commented out line
                x = sin(x);
            ]
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        println!("Output:\n{}", output);
        // Comment is NOT in compiled output (it's stripped for runtime)
        // But the formatter uses the original source which has the comment
        assert!(output.contains("x = sin(x)"), "Code after comment should be preserved");
        assert!(output.contains("Segment 0"), "Seq should have segment 0");
        assert!(output.contains("Segment 1"), "Seq should have segment 1 (raw WGSL)");
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
        let input = "Seq [ Direction(0, -1, 0) | Lm 1; Direction(1, 0, 0) | Lm 1 ]";
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
                Vm 1;
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
        // Line numbers are 0-based offsets from block start (added to block_start_line in caller)
        assert_eq!(err.line, 1);  // Error on second line (index 1)
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

    // Alpha/Am were REMOVED from the language: per-stamp alpha cannot
    // exist while chain coverage is derived from brightness (alpha :=
    // max(rgb)), and after the rgb-rewire they were pure aliases of Bm.
    // They stay in the command gate so use fails loudly as a DSL parse
    // error instead of leaking through as raw WGSL.
    #[test]
    fn test_alpha_removed() {
        assert!(compile_dsl_to_wgsl("Alpha 0").is_err(), "Alpha must be rejected");
        assert!(compile_dsl_to_wgsl("Am 0.5").is_err(), "Am must be rejected");
    }

    #[test]
    fn test_alpha_in_seq_rejected() {
        let input = r#"
            Seq [
                Vm 2 | Lm 1;
                Alpha 0 | Lm 1;
            ]
        "#;
        assert!(compile_dsl_to_wgsl(input).is_err(), "Alpha inside Seq must be rejected");
    }

    #[test]
    fn test_seq_compose_seq_real_case() {
        // Test Seq | Seq Cartesian product
        // Seq [A, B] | Seq [C, D] → Seq [A|C, B|C, A|D, B|D]
        // Right-side items iterate first, then left items within each
        let input = r#"
            Seq [
                Direction(1, 0, 0) | Lm 1;
                Direction(0, 1, 0) | Lm 1;
            ]
            | Seq [
                Vm 1;
                Vm 2;
            ]
        "#;
        let output = compile_dsl_to_wgsl(input).unwrap();
        println!("=== Generated WGSL ===\n{}", output);

        // Cartesian product: 2 items × 2 items = 4 segments
        // Segment 0: Direction(1,0,0) | Vm 1
        // Segment 1: Direction(0,1,0) | Vm 1
        // Segment 2: Direction(1,0,0) | Vm 2
        // Segment 3: Direction(0,1,0) | Vm 2

        // Should have 4 segments
        let segment_count = output.matches("// Segment").count();
        println!("Segment count: {}", segment_count);
        assert_eq!(segment_count, 4, "Should have 4 segments (2 × 2 Cartesian)");

        // Check for velocity multipliers - should see Vm 1 and Vm 2 each applied twice
        let vm1_count = output.matches("velocity = velocity * 1.0").count();
        let vm2_count = output.matches("velocity = velocity * 2.0").count();
        println!("Vm 1 count: {}, Vm 2 count: {}", vm1_count, vm2_count);
        assert_eq!(vm1_count, 2, "Vm 1 should appear twice (once for each left item)");
        assert_eq!(vm2_count, 2, "Vm 2 should appear twice (once for each left item)");

        // Should have both directions appearing twice (once per right item)
        let dir_1_0_0_count = output.matches("vec3<f32>(1.000000, 0.000000, 0.000000)").count();
        let dir_0_1_0_count = output.matches("vec3<f32>(0.000000, 1.000000, 0.000000)").count();
        println!("Direction (1,0,0) count: {}, Direction (0,1,0) count: {}", dir_1_0_0_count, dir_0_1_0_count);
        assert_eq!(dir_1_0_0_count, 2, "Direction (1,0,0) should appear twice");
        assert_eq!(dir_0_1_0_count, 2, "Direction (0,1,0) should appear twice");
    }

    #[test]
    fn test_seq_with_raw_wgsl() {
        // Raw WGSL inside Seq should work
        let input = r#"
            Seq [
                Direction(1, 0, 0) | Lm 1;
                x = sin(x * time * 2);
            ]
        "#;
        let result = compile_dsl_to_wgsl(input);
        match &result {
            Ok(output) => {
                println!("=== Generated WGSL ===\n{}", output);
                assert!(output.contains("// Segment 0"));
                assert!(output.contains("x = sin(x * time * 2)"));
                // Raw WGSL should be time-gated
                assert!(output.contains("if (time >= seg_start) {"));
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Should parse successfully");
            }
        }
    }

    #[test]
    fn test_seq_with_raw_wgsl_user_case() {
        // User's exact case: Direction with Bend, then raw WGSL
        // The raw WGSL should only run DURING its segment (time >= start && time < end)
        let input = r#"
            Seq [
                Direction (1, 0, 0) | Bend (0, -0.1, -0.1, 1) | Lm 8;
                x = sin(x * time * 2);
            ]
        "#;
        let result = compile_dsl_to_wgsl(input);
        match &result {
            Ok(output) => {
                println!("=== Generated WGSL (user case) ===\n{}", output);
                // Segment 0 should have duration 8
                assert!(output.contains("let seg_end = 8.000000"), "First segment should end at 8");
                // Segment 1 (raw WGSL) should start at 8
                assert!(output.contains("let seg_start = 8.000000"), "Second segment should start at 8");
                // Raw WGSL should be time-gated to ONLY run during its segment
                assert!(output.contains("if (time >= seg_start && time < seg_end)"),
                    "Raw WGSL should be time-gated to only run during its segment");
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Should parse successfully");
            }
        }
    }

    #[test]
    fn test_nested_seq_seq_complex() {
        // Test thing3's wgsl block: Seq | Seq with multiple segments
        let input = r#"
            Lm 1/5
            | Sm 9
            | Vm 1
            | Za 1
            | Seq [
                Direction(1, 0, 0)
                | Lm 1
                | Bend (0, 1, 0, 1);
                Direction(1, 0, 0)
                | Lm 1
                | Bend (0, -1, 0, 1);
                Direction(0, 1, 0)
                | Lm 2;
                Direction(1, 0, 0)
                | Lm 2;
                Bm 0.0;
            ]
            | Seq [
                Vm 1;
                Vm 2 | Xa 1;
            ]
        "#;
        let result = compile_dsl_to_wgsl(input);
        match &result {
            Ok(output) => {
                println!("=== Generated WGSL ===\n{}", output);
                // Validate that the WGSL is valid
                let validation = weresocool_ast::wgsl::validate_wgsl(output);
                if let Err(e) = &validation {
                    println!("Validation error: {}", e);
                }
                assert!(validation.is_ok(), "Generated WGSL should be valid");
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Should parse successfully");
            }
        }
    }

    #[test]
    fn test_raw_wgsl_piped_with_direction_and_bend() {
        // This was failing: raw WGSL (y = y + sin(time * 2)) piped with Direction and Bend
        let input = r#"
            Seq [
                Direction (1, 0, 0)
                | y = y + sin(time * 2)
                | Bend (0, -0.1, -0.1, 1) | Lm 2;
                Direction (1, 0, 0)
                | y = y + sin(time * 2);
            ]
        "#;
        let result = compile_dsl_to_wgsl(input);
        match &result {
            Ok(output) => {
                println!("=== Generated WGSL ===\n{}", output);
                assert!(output.contains("y = y + sin(time * 2)"), "Raw WGSL should be included");
                // Validate that the WGSL is valid
                let validation = weresocool_ast::wgsl::validate_wgsl(output);
                if let Err(e) = &validation {
                    println!("Validation error: {}", e);
                }
                assert!(validation.is_ok(), "Generated WGSL should be valid");
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Should parse successfully");
            }
        }
    }

    #[test]
    fn test_different_raw_wgsl_per_seq_segment() {
        // Each segment should have its own raw WGSL - segment 1 should NOT have sin
        let input = r#"
            Seq [
                Direction (1, 0, 0)
                | y = y + sin(time * 2)
                | Bend (0, -0.1, -0.1, 1) | Lm 2;
                Direction (1, 0, 0)
                | y = y * 2;
            ]
        "#;
        let result = compile_dsl_to_wgsl(input);
        match &result {
            Ok(output) => {
                println!("=== Generated WGSL ===\n{}", output);

                // Should have sin in segment 0
                assert!(output.contains("y = y + sin(time * 2)"), "Segment 0 should have sin");
                // Should have y * 2 in segment 1
                assert!(output.contains("y = y * 2"), "Segment 1 should have y * 2");

                // Count occurrences - sin should appear only in segment 0
                let sin_count = output.matches("y = y + sin(time * 2)").count();
                let mult_count = output.matches("y = y * 2").count();
                println!("sin count: {}, mult count: {}", sin_count, mult_count);

                // Each should appear exactly once (in their respective segments)
                assert_eq!(sin_count, 1, "sin should appear exactly once (in segment 0)");
                assert_eq!(mult_count, 1, "y * 2 should appear exactly once (in segment 1)");

                // Validate that the WGSL is valid
                let validation = weresocool_ast::wgsl::validate_wgsl(output);
                if let Err(e) = &validation {
                    println!("Validation error: {}", e);
                }
                assert!(validation.is_ok(), "Generated WGSL should be valid");
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Should parse successfully");
            }
        }
    }
}

    #[test]
    fn test_am0_sm0_segments() {
        let input = r#"
            Seq [
                Bm 1
                | Direction (1, 0, 0)
                | Bend (0, -0.1, -0.1, 1) | Lm 4;
                Direction (1, 0, 0) | Lm 3;
                Bm 0
                | Sm 0 | Lm 2;
            ]
            | Seq [Ya 0; Ya 2]
        "#;
        let result = compile_dsl_to_wgsl(input);
        assert!(result.is_ok(), "Should parse: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Am0 Sm0 Test ===\n{}", output);
        
        // Should have 6 segments (3 items × 2 Ya items)
        let segment_count = output.matches("// Segment").count();
        println!("Segment count: {}", segment_count);
        assert_eq!(segment_count, 6, "Should have 6 segments (3 × 2 Cartesian)");
    }

    #[test]
    fn test_wgsl_expression_in_sm() {
        // Test WGSL expression as value: Sm time
        let result = compile_dsl_to_wgsl("Sm time");
        assert!(result.is_ok(), "Should parse Sm with expression: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Sm time ===\n{}", output);
        assert!(output.contains("scale = scale * time"), "Should have scale = scale * time");
    }

    #[test]
    fn test_wgsl_expression_function_call() {
        // Test function call as value: Ya sin(time)
        let result = compile_dsl_to_wgsl("Ya sin(time)");
        assert!(result.is_ok(), "Should parse Ya with function call: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Ya sin(time) ===\n{}", output);
        assert!(output.contains("y = y + sin(time)"), "Should have y = y + sin(time)");
    }

    #[test]
    fn test_wgsl_expression_binary() {
        // Test binary expression: Sm time * 2
        let result = compile_dsl_to_wgsl("Sm time * 2");
        assert!(result.is_ok(), "Should parse Sm with binary expression: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Sm time * 2 ===\n{}", output);
        assert!(output.contains("scale = scale * time * 2"), "Should have scale = scale * time * 2");
    }

    #[test]
    fn test_wgsl_expression_in_direction() {
        // Test expressions in Direction: Direction(sin(time), cos(time), 0)
        let result = compile_dsl_to_wgsl("Direction(sin(time), cos(time), 0)");
        assert!(result.is_ok(), "Should parse Direction with expressions: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Direction with expressions ===\n{}", output);
        assert!(output.contains("sin(time)"), "Should have sin(time) in direction");
        assert!(output.contains("cos(time)"), "Should have cos(time) in direction");
    }

    #[test]
    fn test_wgsl_expression_with_literals() {
        // Test mixing expressions and literals
        let result = compile_dsl_to_wgsl("Direction(1, sin(time), 0) | Sm 2 | Vm progress");
        assert!(result.is_ok(), "Should parse mixed expression/literal: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Mixed expression/literal ===\n{}", output);
        assert!(output.contains("1.000000"), "Should have literal 1 in direction");
        assert!(output.contains("sin(time)"), "Should have sin(time) in direction");
        assert!(output.contains("scale = scale * 2.000000"), "Should have scale * 2");
        assert!(output.contains("velocity = velocity * progress"), "Should have velocity * progress");
    }

    #[test]
    fn test_quoted_expression_number_first() {
        // Quoted string allows number-first expressions
        let result = compile_dsl_to_wgsl(r#"Sm "2 * time""#);
        assert!(result.is_ok(), "Should parse quoted number-first: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Quoted number-first ===\n{}", output);
        assert!(output.contains("scale = scale * 2 * time"), "Should have 2 * time");
    }

    #[test]
    fn test_quoted_expression_nested_parens() {
        // Quoted string allows nested function calls
        let result = compile_dsl_to_wgsl(r#"Sm "sin(cos(time))""#);
        assert!(result.is_ok(), "Should parse quoted nested parens: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Quoted nested parens ===\n{}", output);
        assert!(output.contains("scale = scale * sin(cos(time))"), "Should have sin(cos(time))");
    }

    #[test]
    fn test_quoted_expression_negative() {
        // Quoted string allows negative identifiers
        let result = compile_dsl_to_wgsl(r#"Sm "-time""#);
        assert!(result.is_ok(), "Should parse quoted negative: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Quoted negative ===\n{}", output);
        assert!(output.contains("scale = scale * -time"), "Should have -time");
    }

    #[test]
    fn test_quoted_expression_grouping_parens() {
        // Quoted string allows grouping parentheses
        let result = compile_dsl_to_wgsl(r#"Sm "(x + y) * 0.5""#);
        assert!(result.is_ok(), "Should parse quoted grouping: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Quoted grouping ===\n{}", output);
        assert!(output.contains("scale = scale * (x + y) * 0.5"), "Should have (x + y) * 0.5");
    }

    #[test]
    fn test_quoted_in_direction_with_comma() {
        // Quoted string in Direction allows commas inside functions
        let result = compile_dsl_to_wgsl(r#"Direction("max(x, y)", 0, 0)"#);
        assert!(result.is_ok(), "Should parse quoted with comma: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Quoted with comma ===\n{}", output);
        assert!(output.contains("max(x, y)"), "Should have max(x, y)");
    }

    #[test]
    fn color_law_rejects_absolute_set_but_allows_relative() {
        // Absolute constant set on a palette channel → rejected.
        for bad in ["red = 0.5", "green = 1", "blue = -0.2"] {
            let r = compile_dsl_to_wgsl(bad);
            assert!(r.is_err(), "`{bad}` should be rejected, got {:?}", r.ok());
        }
        // Relative forms + DSL color ops pass.
        for ok in ["red = red * 0.5", "green = green + 0.1", "blue = blue * fade", "Bm 0.5", "Ba 0.1"] {
            let r = compile_dsl_to_wgsl(ok);
            assert!(r.is_ok(), "`{ok}` should pass, got {:?}", r.err());
        }
        // Expression RHS is left alone (only the constant footgun is caught) —
        // e.g. rainforest's animated `red = 0.5 + sin(time*4)*0.5`.
        assert!(compile_dsl_to_wgsl("red = some_var * 0.5").is_ok());
        assert!(compile_dsl_to_wgsl("red = 0.5 + sin(time * 4.0) * 0.5").is_ok());
        // `alpha` is derived downstream, not guarded.
        assert!(compile_dsl_to_wgsl("alpha = 0.3").is_ok());
    }

    #[test]
    fn test_rotation_ry() {
        // Test Ry rotation (spin around Y axis)
        let result = compile_dsl_to_wgsl("Ry 1");
        assert!(result.is_ok(), "Should parse Ry: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Ry 1 ===\n{}", output);
        assert!(output.contains("Global rotation"), "Should have rotation comment");
        assert!(output.contains("angle_y"), "Should have angle_y");
        assert!(output.contains("6.28318"), "Should convert to radians");
    }

    #[test]
    fn test_rotation_all_axes() {
        // Test all rotation axes composed
        let result = compile_dsl_to_wgsl("Rx 0.25 | Ry 0.5 | Rz 0.1");
        assert!(result.is_ok(), "Should parse all rotation axes: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Rx | Ry | Rz ===\n{}", output);
        assert!(output.contains("angle_x"), "Should have angle_x");
        assert!(output.contains("angle_y"), "Should have angle_y");
        assert!(output.contains("angle_z"), "Should have angle_z");
    }

    #[test]
    fn test_rotation_in_seq() {
        // Test rotation in a sequence
        let result = compile_dsl_to_wgsl(r#"
            Seq [
                Ry 0.25 | Lm 1;
                Ry 0.5 | Lm 1;
            ]
        "#);
        assert!(result.is_ok(), "Should parse rotation in seq: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Rotation in Seq ===\n{}", output);
        assert!(output.contains("// Segment 0"), "Should have segment 0");
        assert!(output.contains("// Segment 1"), "Should have segment 1");
        assert!(output.contains("angle_y"), "Should have rotation in segments");
    }

    #[test]
    fn test_rotation_with_expression() {
        // Test rotation with WGSL expression
        let result = compile_dsl_to_wgsl("Ry time");
        assert!(result.is_ok(), "Should parse Ry with expression: {:?}", result.err());
        let output = result.unwrap();
        println!("=== Ry time ===\n{}", output);
        assert!(output.contains("angle_y = (time) * 6.28318"), "Should have time expression");
    }
