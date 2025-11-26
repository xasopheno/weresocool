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
use weresocool_error::{Error, ParseError};
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

    // TODO: Is this correct?
    for (scope_name, scope) in defs.ops.clone().iter_mut() {
        for (name, term) in scope {
            match term {
                Term::Nf(nf) => {
                    result.ops.insert(scope_name, name, Term::Nf(nf.to_owned()));
                }
                Term::Op(op) => {
                    let mut nf = NormalForm::init();
                    op.apply_to_normal_form(&mut nf, &mut defs)?;

                    result.ops.insert(scope_name, name, Term::Nf(nf));
                }
                Term::FunDef(fun) => {
                    result.ops.insert(scope_name, name, Term::FunDef(fun.to_owned()));
                }
                Term::Lop(lop) => {
                    let mut nf = NormalForm::init();
                    lop.apply_to_normal_form(&mut nf, &mut defs.clone())?;
                    result.ops.insert(scope_name, name, Term::Nf(nf));
                }
                Term::Gen(generator) => {
                    let mut nf = NormalForm::init();
                    generator.apply_to_normal_form(&mut nf, &mut defs.clone())?;

                    result.ops.insert(scope_name, name, Term::Nf(nf));
                }
            };
        }
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
// Returns the processed string and a SourceMap for error position mapping
// Fails fast on the first WGSL validation error
pub fn process_wgsl_blocks(composition: &str, defs: &mut Defs, skip_validation: bool) -> Result<(String, SourceMap), Error> {
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

                // Print the error with colored output (same style as WGSL errors)
                println!("\n");
                e.display_colored(composition, actual_line, actual_column);

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
                // Print the error with colored output
                println!("\n");
                e.display_colored(composition);
                return Err(ParseError {
                    message: format!("WGSL error: {}", e.message),
                    line: e.line,
                    column: e.column,
                }
                .into_error());
            }
        }

        // Insert the compiled WGSL code and get its ID
        let id = defs.wgsl.insert(wgsl_code.to_string());

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
// This wrapper calls the new function with skip_validation set to false
pub fn process_wgsl_blocks_with_validation(composition: &str, defs: &mut Defs) -> Result<(String, SourceMap), Error> {
    process_wgsl_blocks(composition, defs, false)
}

/// Parse source code to raw AST without evaluating to NormalForm.
/// This is useful for formatting where we want to preserve the source structure.
pub fn parse_to_raw_ast(
    vec_string: Vec<String>,
) -> Result<ParsedComposition, Error> {
    let mut defs: Defs = Default::default();

    let (imports_needed, composition) = handle_whitespace_and_imports(vec_string)?;

    // For formatting, we skip imports and just parse the current file
    if !imports_needed.is_empty() {
        // TODO: Handle imports for formatting
    }

    // Process WGSL blocks - extract them and replace with IDs
    let (processed_composition, source_map) = process_wgsl_blocks(&composition, &mut defs, true)?;

    let init = socool::SoCoolParser::new().parse(&mut defs, &processed_composition);
    match init {
        Ok(init) => {
            // Don't process to NormalForm - keep raw AST
            // Just copy WGSL and colors
            let mut result_defs = defs.clone();

            if let Some(background_color) = init.background_color.clone() {
                result_defs.colors.insert_by_name("background_color".to_string(), background_color);
            }

            Ok(ParsedComposition { init, defs: result_defs })
        }
        Err(error) => {
            println!("\n");
            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            let (line, column) = handle_parse_error(location, &composition, &source_map);

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

    let (imports_needed, composition) = handle_whitespace_and_imports(vec_string)?;
    
    // Process WGSL blocks - extract them and replace with IDs
    // This validates each WGSL block and fails fast on the first error
    let (processed_composition, source_map) = process_wgsl_blocks(&composition, &mut defs, false)?;
    
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

    let init = socool::SoCoolParser::new().parse(&mut defs, &processed_composition);
    match init {
        Ok(init) => {
            let mut result_defs = process_op_table(&mut defs)?;

            // Ensure WGSL blocks and colors are preserved in the final result
            result_defs.wgsl = defs.wgsl.clone();
            result_defs.colors = defs.colors.clone();

            if let Some(background_color) = init.background_color.clone() {
                result_defs.colors.insert_by_name("background_color".to_string(), background_color);
            }

            Ok(ParsedComposition { init, defs: result_defs })
        }
        Err(error) => {
            println!("\n");
            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            let (line, column) = handle_parse_error(location, &composition, &source_map);

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
    for line in lines {
        let l = line;
        let copy_l = l.trim_start();
        if copy_l.starts_with("--") {
            composition.push_str("\n");
        } else if is_import(copy_l.to_string()) {
            imports_needed.push(copy_l.to_owned());
            composition.push_str("\n");
        } else {
            composition.push_str("\n");
            composition.push_str(&l);
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
