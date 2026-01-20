lalrpop_mod!(pub socool);
use crate::error_handling::handle_parse_error;
use crate::imports::{get_filepath_and_import_name, is_import};
use colored::*;
use num_rational::Rational64;
use path_clean::PathClean;
use weresocool_ast::color::ColorValue;
use weresocool_ast::{Defs, NormalForm, Normalize, Op, Term};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use weresocool_error::{ColorError, Error, ParseError};
use weresocool_shared::timing_print;
use regex;

/// Tracks offset adjustments when WGSL blocks are replaced with tokens.
/// Used to map error positions back to the original source.
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    /// Sorted list of (processed_pos, cumulative_delta)
    /// where cumulative_delta = original_pos - processed_pos at that point
    adjustments: Vec<(usize, isize)>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { adjustments: vec![] }
    }

    /// Record a replacement: original text of `original_len` was replaced with text of `replacement_len`
    /// at position `processed_pos` in the processed string.
    pub fn add_replacement(&mut self, processed_pos: usize, original_len: usize, replacement_len: usize) {
        let delta = (original_len as isize) - (replacement_len as isize);
        let prev_delta = self.adjustments.last().map(|(_, d)| *d).unwrap_or(0);
        self.adjustments.push((processed_pos, prev_delta + delta));
    }

    /// Convert a position in the processed string back to the original source position.
    pub fn to_original(&self, processed_pos: usize) -> usize {
        let delta = self.adjustments
            .iter()
            .take_while(|(pos, _)| *pos <= processed_pos)
            .last()
            .map(|(_, d)| *d)
            .unwrap_or(0);
        (processed_pos as isize + delta) as usize
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Init {
    pub f: Rational64,
    pub l: Rational64,
    pub g: Rational64,
    pub p: Rational64,
    pub background_color: Option<ColorValue>
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParsedComposition {
    pub init: Init,
    pub defs: Defs,
}

fn process_op_table(mut defs: &mut Defs) -> Result<Defs, Error> {
    let mut result: Defs = Defs::default();
    result.colors = defs.colors.clone();

    let total_start = std::time::Instant::now();
    let mut op_count = 0;
    let mut nf_count = 0;
    let mut slowest_op: Option<(String, std::time::Duration)> = None;

    // Collect all (scope_name, name, term) tuples first to avoid borrow issues
    let entries: Vec<(String, String, Term)> = defs.ops.clone()
        .iter()
        .flat_map(|(scope_name, scope)| {
            scope.iter().map(move |(name, term)| {
                (scope_name.clone(), name.clone(), term.clone())
            })
        })
        .collect();

    for (scope_name, name, term) in entries {
        let op_start = std::time::Instant::now();
        match term {
            Term::Nf(nf) => {
                result.ops.insert(&scope_name, &name, Term::Nf(nf.to_owned()));
                nf_count += 1;
            }
            Term::Op(op) => {
                let mut nf = NormalForm::init();
                op.apply_to_normal_form(&mut nf, &mut defs)?;
                let elapsed = op_start.elapsed();
                if elapsed > std::time::Duration::from_millis(100) {
                    timing_print!("[process_op_table] Op '{}' took {:?}", name, elapsed);
                }
                if slowest_op.as_ref().map_or(true, |(_, d)| elapsed > *d) {
                    slowest_op = Some((name.clone(), elapsed));
                }
                // MEMOIZATION: Update defs so subsequent lookups get the normalized form
                defs.ops.insert(&scope_name, &name, Term::Nf(nf.clone()));
                result.ops.insert(&scope_name, &name, Term::Nf(nf));
                op_count += 1;
            }
            Term::FunDef(fun) => {
                result.ops.insert(&scope_name, &name, Term::FunDef(fun.to_owned()));
            }
            Term::Lop(lop) => {
                let mut nf = NormalForm::init();
                lop.apply_to_normal_form(&mut nf, &mut defs.clone())?;
                // MEMOIZATION: Update defs for Lop too
                defs.ops.insert(&scope_name, &name, Term::Nf(nf.clone()));
                result.ops.insert(&scope_name, &name, Term::Nf(nf));
            }
            Term::Gen(generator) => {
                let mut nf = NormalForm::init();
                generator.apply_to_normal_form(&mut nf, &mut defs.clone())?;
                // MEMOIZATION: Update defs for Gen too
                defs.ops.insert(&scope_name, &name, Term::Nf(nf.clone()));
                result.ops.insert(&scope_name, &name, Term::Nf(nf));
            }
        };
    }

    timing_print!("[process_op_table] Total: {:?} ({} ops, {} pre-normalized)", total_start.elapsed(), op_count, nf_count);
    if let Some((name, duration)) = slowest_op {
        timing_print!("[process_op_table] Slowest op: '{}' took {:?}", name, duration);
    }

    result.ops.stems = defs.ops.stems.to_owned();

    Ok(result)
}
pub fn read_file(filename: &str) -> Result<File, Error> {
    let f = File::open(filename);
    match f {
        Ok(f) => return Ok(f),
        _ => {
            println!(
                "{} {}\n",
                "\n        File not found:".red().bold(),
                filename.red().bold()
            );

            return Err(Error::with_msg(format!("File not found: {}", filename)));
        }
    };
}

pub fn filename_to_vec_string(filename: &str) -> Result<Vec<String>, Error> {
    let file = read_file(filename)?;
    let reader = BufReader::new(&file);
    Ok(reader
        .lines()
        .map(|line| line)
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn language_to_vec_string(language: &str) -> Vec<String> {
    language.split('\n').map(|l| l.to_string()).collect()
}

/// Helper to count lines (1-based) before a given byte offset
/// Note: composition is built with a leading \n for each line, so newline count equals original line number
fn count_lines_before(text: &str, offset: usize) -> usize {
    text[..offset].chars().filter(|&c| c == '\n').count()
}

// Extract WGSL code blocks from source code and replace with IDs
// If skip_validation is true, validation will be skipped (used in tests)
// If quiet is true, error display is suppressed (used in formatter)
// Returns the processed string and a SourceMap for error position mapping
// Fails fast on the first WGSL validation error
pub fn process_wgsl_blocks(composition: &str, defs: &mut Defs, skip_validation: bool, quiet: bool) -> Result<(String, SourceMap), Error> {
    let mut result = String::new();
    let mut source_map = SourceMap::new();

    // Extract WGSL blocks with regex pattern that's flexible with whitespace
    // This pattern matches:
    // 1. WGSL keyword (case-insensitive) followed by optional whitespace and an opening brace
    // 2. The content inside (including any nested braces)
    // 3. The closing brace
    // The (?:\{[^{}]*\}[^{}]*)* part handles nested braces like if-else blocks
    let regex_pattern = r"(?i)wgsl\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}";
    let re = regex::Regex::new(regex_pattern).unwrap();

    let mut last_end = 0;
    let mut processed_pos = 0;

    // Find all WGSL blocks and replace them with tokens
    for cap in re.captures_iter(composition) {
        let full_match = cap.get(0).unwrap();
        let wgsl_capture = cap.get(1).unwrap().as_str();
        let raw_wgsl_code = wgsl_capture.trim();

        // Copy text before this match
        let before_match = &composition[last_end..full_match.start()];
        result.push_str(before_match);
        processed_pos += before_match.len();

        // Compile DSL syntax to WGSL (if any DSL commands are present)
        let wgsl_content_start = cap.get(1).unwrap().start();
        let block_start_line = count_lines_before(composition, wgsl_content_start);

        let wgsl_code = match crate::wgsl_dsl::compile_dsl_to_wgsl(raw_wgsl_code) {
            Ok(compiled) => compiled,
            Err(e) => {
                // Calculate the actual line in original source
                let actual_line = block_start_line + e.line;

                // Calculate the correct column in the original source
                // The DSL column is relative to the line in raw_wgsl_code (which has first line trimmed)
                // We need to find the actual indentation in the original composition
                let lines: Vec<&str> = composition.split('\n').collect();
                let actual_column = if actual_line < lines.len() {
                    let orig_line = lines[actual_line];
                    let orig_leading_ws = orig_line.len() - orig_line.trim_start().len();

                    // For line 1 of the block, raw_wgsl_code has no leading whitespace
                    // For other lines, raw_wgsl_code preserves the indentation
                    if e.line == 1 {
                        // First line: DSL column is relative to trimmed content
                        // Need to add back the original indentation
                        orig_leading_ws + e.column
                    } else {
                        // Other lines: DSL already accounts for leading_ws in the line
                        e.column
                    }
                } else {
                    e.column
                };

                // Print colored error output
                e.display_colored(composition, actual_line, actual_column, quiet);

                return Err(ParseError {
                    message: e.message.clone(),
                    line: actual_line,
                    column: actual_column,
                }
                .into_error());
            }
        };

        // Validate the WGSL code (unless skipped for tests)
        // Fail fast: return immediately on first error
        if !skip_validation {
            // Calculate the line number where this WGSL block's content starts
            // (the line after "wgsl {")
            let wgsl_content_start = cap.get(1).unwrap().start();
            let block_start_line = count_lines_before(composition, wgsl_content_start);

            if let Err(e) = weresocool_ast::wgsl::validate_wgsl_with_position(
                &wgsl_code,
                block_start_line,
                composition,
            ) {
                // Print colored error output
                e.display_colored(composition, quiet);

                return Err(ParseError {
                    message: format!("WGSL error: {}", e.message),
                    line: e.line,
                    column: e.column,
                }
                .into_error());
            }
        }

        // Insert the compiled WGSL code and get its ID
        // Store both compiled (for runtime) and original (for formatting)
        let id = defs.wgsl.insert(wgsl_code.to_string(), raw_wgsl_code.to_string());

        // Replace the WGSL block with a token the parser can recognize
        // Using @WGSL@ prefix to clearly distinguish from regular identifiers
        let token = format!("@WGSL@{}", id);
        result.push_str(&token);

        // Record the adjustment: original length vs token length
        let original_len = full_match.end() - full_match.start();
        source_map.add_replacement(processed_pos, original_len, token.len());

        processed_pos += token.len();
        last_end = full_match.end();
    }

    // Copy remaining text after last match
    result.push_str(&composition[last_end..]);

    Ok((result, source_map))
}

// For backwards compatibility with existing code
// This wrapper calls the new function with skip_validation=false, quiet=false
pub fn process_wgsl_blocks_with_validation(composition: &str, defs: &mut Defs) -> Result<(String, SourceMap), Error> {
    process_wgsl_blocks(composition, defs, false, false)
}

/// Result from parsing for formatting - includes source for span extraction
#[derive(Clone, Debug)]
pub struct FormatParseResult {
    pub composition: ParsedComposition,
    /// Original source (with WGSL blocks intact)
    pub source: String,
    /// Processed source (WGSL replaced with @WGSL@N tokens) - spans match this
    pub processed_source: Option<String>,
}

/// Parse source code to raw AST without evaluating to NormalForm.
/// This is useful for formatting where we want to preserve the source structure.
pub fn parse_to_raw_ast(
    vec_string: Vec<String>,
) -> Result<ParsedComposition, Error> {
    parse_for_format(vec_string).map(|r| r.composition)
}

/// Parse source code for formatting - returns AST plus original source.
/// The source can be used to extract original text via spans.
///
/// IMPORTANT: For formatting, we use the processed source (with WGSL tokens replaced)
/// because that's what the parser sees and generates spans for. The original WGSL
/// content is stored in defs.wgsl and can be looked up by ID.
pub fn parse_for_format(
    vec_string: Vec<String>,
) -> Result<FormatParseResult, Error> {
    parse_for_format_inner(vec_string, true)  // quiet=true for formatter
}

fn parse_for_format_inner(
    vec_string: Vec<String>,
    quiet: bool,
) -> Result<FormatParseResult, Error> {
    let mut defs: Defs = Default::default();

    let (imports_needed, composition) = handle_whitespace_and_imports(vec_string)?;

    // For formatting, we skip imports and just parse the current file
    if !imports_needed.is_empty() {
        // TODO: Handle imports for formatting
    }

    // Process WGSL blocks - extract them and replace with IDs
    // We use the PROCESSED source for spans because that's what the parser operates on
    // skip_validation=true for formatting (we just want to format, not validate WGSL)
    let (processed_composition, source_map) = process_wgsl_blocks(&composition, &mut defs, true, quiet)?;

    let init = socool::SoCoolParser::new().parse(&mut defs, &processed_composition);
    match init {
        Ok(init) => {
            // Don't process to NormalForm - keep raw AST
            // Just copy WGSL and colors
            let mut result_defs = defs.clone();

            if let Some(background_color) = init.background_color.clone() {
                result_defs.colors.insert_by_name("background_color".to_string(), background_color);
            }

            Ok(FormatParseResult {
                composition: ParsedComposition { init, defs: result_defs },
                // Return ORIGINAL source - we'll need to map spans or fall back to AST formatting
                // when spans don't match (e.g., when WGSL processing changed positions)
                source: composition,
                // Also return processed source for span-accurate lookups
                processed_source: Some(processed_composition),
            })
        }
        Err(error) => {
            if !quiet {
                eprintln!("\n");
            }

            // Check if this is a color error (format: "location:colorname")
            if let lalrpop_util::ParseError::User { error: err_str } = &error {
                if let Some((loc_str, bad_color)) = err_str.split_once(':') {
                    if let Ok(loc) = loc_str.parse::<usize>() {
                        let start = source_map.to_original(loc);
                        // Calculate line and column from position
                        let mut line: usize = 0;
                        let mut column: usize = 0;
                        for (n_c, c) in composition.chars().enumerate() {
                            if n_c >= start {
                                break;
                            }
                            if c == '\n' {
                                line += 1;
                                column = 0;
                            } else {
                                column += 1;
                            }
                        }

                        // Display error
                        if !quiet {
                            weresocool_error::ErrorDisplay {
                                source: &composition,
                                line,
                                column,
                                label: &format!("Invalid color '{}'", bad_color),
                                use_cyan: false,
                            }.display(false);
                        }

                        return Err(ColorError {
                            color: bad_color.to_string(),
                            line,
                            column,
                        }
                        .into_error());
                    }
                }
            }

            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            let (line, column) = handle_parse_error(location, &composition, &source_map, quiet);

            Err(ParseError {
                message: "Unexpected Token".to_string(),
                line,
                column,
            }
            .into_error())
        }
    }
}

pub fn parse_file(
    vec_string: Vec<String>,
    prev_defs: Option<Defs>,
    working_path: Option<PathBuf>,
) -> Result<ParsedComposition, Error> {
    let mut defs: Defs = if let Some(defs) = prev_defs {
        defs
    } else {
        Default::default()
    };

    let ws_start = std::time::Instant::now();
    let (imports_needed, composition) = handle_whitespace_and_imports(vec_string)?;
    timing_print!("[parse_file] handle_whitespace_and_imports: {:?}", ws_start.elapsed());

    // Process WGSL blocks - extract them and replace with IDs
    // This validates each WGSL block and fails fast on the first error
    // quiet=false to show errors during actual parsing
    let wgsl_start = std::time::Instant::now();
    let (processed_composition, source_map) = process_wgsl_blocks(&composition, &mut defs, false, false)?;
    timing_print!("[parse_file] process_wgsl_blocks: {:?}", wgsl_start.elapsed());

    for import in imports_needed {
        let (mut filepath, import_name) = get_filepath_and_import_name(import);
        if let Some(mut wd) = working_path.clone() {
            wd.push(filepath);
            filepath = wd.clean().display().to_string();
        }
        // dbg!(&filepath);
        let vec_string = filename_to_vec_string(&filepath.to_string())?;
        let parsed_composition = parse_file(vec_string, Some(defs.clone()), working_path.clone())?;

        // Merge WGSL blocks from imported files
        for (id, code) in &parsed_composition.defs.wgsl.map {
            defs.wgsl.map.insert(*id, code.clone());
        }
        // Update next_id to ensure subsequent imports get unique IDs
        defs.wgsl.update_next_id(parsed_composition.defs.wgsl.next_id());

        // Merge colors from imported files
        for (name, value) in parsed_composition.defs.colors.map.iter() {
            defs.colors.map.insert(name.clone(), value.clone());
        }
        // Update next_id to ensure subsequent imports get unique IDs
        defs.colors.update_next_id(parsed_composition.defs.colors.next_id());

        for (scope_name, scope) in parsed_composition.defs.ops.iter() {
            for (n, term) in scope {
                let mut name = import_name.clone();
                name.push('.');
                name.push_str(n);
                defs.ops.insert(scope_name, name, term.clone());
            }
        }
    }

    let parse_start = std::time::Instant::now();
    let init = socool::SoCoolParser::new().parse(&mut defs, &processed_composition);
    timing_print!("[parse_file] SoCoolParser::parse: {:?}", parse_start.elapsed());

    match init {
        Ok(init) => {
            let op_table_start = std::time::Instant::now();
            let mut result_defs = process_op_table(&mut defs)?;
            timing_print!("[parse_file] process_op_table: {:?}", op_table_start.elapsed());

            // Ensure WGSL blocks and colors are preserved in the final result
            result_defs.wgsl = defs.wgsl.clone();
            result_defs.colors = defs.colors.clone();

            if let Some(background_color) = init.background_color.clone() {
                result_defs.colors.insert_by_name("background_color".to_string(), background_color);
            }

            Ok(ParsedComposition { init, defs: result_defs })
        }
        Err(error) => {
            eprintln!("\n");

            // Check if this is a color error (format: "location:colorname")
            if let lalrpop_util::ParseError::User { error: err_str } = &error {
                if let Some((loc_str, bad_color)) = err_str.split_once(':') {
                    if let Ok(loc) = loc_str.parse::<usize>() {
                        let start = source_map.to_original(loc);
                        // Calculate line and column from position
                        let mut line: usize = 0;
                        let mut column: usize = 0;
                        for (n_c, c) in composition.chars().enumerate() {
                            if n_c >= start {
                                break;
                            }
                            if c == '\n' {
                                line += 1;
                                column = 0;
                            } else {
                                column += 1;
                            }
                        }

                        // Display error
                        weresocool_error::ErrorDisplay {
                            source: &composition,
                            line,
                            column,
                            label: &format!("Invalid color '{}'", bad_color),
                            use_cyan: false,
                        }.display(false);

                        return Err(ColorError {
                            color: bad_color.to_string(),
                            line,
                            column,
                        }
                        .into_error());
                    }
                }
            }

            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            let (line, column) = handle_parse_error(location, &composition, &source_map, false);

            Err(ParseError {
                message: "Unexpected Token".to_string(),
                line,
                column,
            }
            .into_error())
        }
    }
}

fn handle_whitespace_and_imports(lines: Vec<String>) -> Result<(Vec<String>, String), Error> {
    let mut composition = String::new();
    let mut imports_needed: Vec<String> = vec![];
    let mut first_content = true;

    for line in lines.into_iter() {
        let l = line;
        let copy_l = l.trim_start();

        if is_import(copy_l.to_string()) {
            imports_needed.push(copy_l.to_owned());
            // Add newline placeholder for import lines (but not before first content)
            if !first_content {
                composition.push_str("\n");
            }
        } else {
            // Don't add newline before the first line of content
            if !first_content {
                composition.push_str("\n");
            }
            composition.push_str(&l);
            first_content = false;
        }
    }

    Ok((imports_needed, composition))
}

pub fn handle_fit_length_recursively(terms: Vec<Term>) -> Vec<Term> {
    let mut result = vec![];
    let mut i = 0;

    for term in terms.iter() {
        if i == 0 {
            result.push(term.to_owned());
        } else {
            match term {
                Term::Op(op) => match op {
                    Op::WithLengthRatioOf { with_length_of, .. } => {
                        let op1 = Term::Op(Op::Compose {
                            operations: result[0..i].into_iter().cloned().collect(),
                        });
                        result = vec![Term::Op(Op::Compose {
                            operations: vec![
                                op1.to_owned(),
                                Term::Op(Op::WithLengthRatioOf {
                                    with_length_of: with_length_of.clone(),
                                    main: Some(Box::new(op1)),
                                }),
                            ],
                        })];
                        i = 0;
                    }
                    _ => result.push(term.to_owned()),
                },
                _ => result.push(term.to_owned()),
            }
        }
        i += 1;
    }

    result
}

pub fn handle_repeat_recursively(terms: Vec<Term>) -> Vec<Term> {
    let mut result = vec![];
    let mut i = 0;

    for term in terms.iter() {
        match term {
            Term::Op(Op::Sequence { operations, .. }) => {
                // Check if this Sequence is all AsIs operations (i.e., a Repeat)
                let all_asis = operations.iter().all(|op| {
                    matches!(op, Term::Op(Op::AsIs))
                });

                if all_asis && !operations.is_empty() && i > 0 {
                    // This is a Repeat! Take all previous operations and wrap them
                    let count = operations.len() as i64;
                    let ops_to_repeat = result.drain(0..i).collect();

                    result.push(Term::Op(Op::Repeat {
                        operations: ops_to_repeat,
                        count,
                    }));

                    // Reset index since we collapsed everything into a Repeat
                    i = 1;
                } else {
                    // Normal Sequence, just add it
                    result.push(term.to_owned());
                    i += 1;
                }
            }
            _ => {
                result.push(term.to_owned());
                i += 1;
            }
        }
    }

    result
}

mod tests {
    #[test]
    fn filename_and_language_to_vec_string() {
        use super::*;
        let filename = "./working.socool";
        let mut language = "".to_string();
        let f = File::open(filename).expect("couldn't open ./working.socool");
        let file = BufReader::new(&f);
        file.lines().for_each(|line| {
            let l = line.expect("Could not parse line");
            language.push_str(&l);
            language.push_str("\n");
        });

        let from_filename = filename_to_vec_string(filename).unwrap();
        let from_language = language_to_vec_string(language.as_str());

        for (a, b) in from_filename.iter().zip(&from_language) {
            assert_eq!(a, b);
        }
    }
}
