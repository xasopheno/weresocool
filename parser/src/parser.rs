lalrpop_mod!(pub socool);
use crate::error_handling::{handle_parse_error, ExtractedParseError};
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
use weresocool_error::{ColorError, Error, ParseError};
use weresocool_shared::{timing_now, timing_print};
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
    pub background_color: Option<ColorValue>,
    /// THE VISUAL BASIS — half-width and half-height of the frame, in the
    /// same spirit as `f`, `l`, `g` and `p` are the audio basis. A piece that
    /// declares `w: 16/9, h: 1` is saying where its edges are, so a visual
    /// op can name a PLACE: `Xa(1)` is the right edge, whatever the render
    /// resolution happens to be.
    ///
    /// Optional, and absent in every piece written before it existed — when
    /// it is absent the renderer keeps deriving the frame from resolution,
    /// exactly as it always has.
    pub w: Option<Rational64>,
    pub h: Option<Rational64>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParsedComposition {
    pub init: Init,
    pub defs: Defs,
}

fn process_op_table(mut defs: &mut Defs) -> Result<Defs, Error> {
    let mut result: Defs = Defs::default();
    result.colors = defs.colors.clone();

    let total_start = timing_now!();
    let mut op_count = 0;
    let mut nf_count = 0;
    #[cfg(not(target_arch = "wasm32"))]
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
        let op_start = timing_now!();
        match term {
            Term::Nf(nf) => {
                result.ops.insert(&scope_name, &name, Term::Nf(nf.to_owned()));
                nf_count += 1;
            }
            Term::Op(op) => {
                let mut nf = NormalForm::init();
                op.apply_to_normal_form(&mut nf, &mut defs)?;
                // Slowest-op tracking uses `elapsed` directly (not via timing_print!),
                // so it must be cfg-gated — std::time isn't available on wasm32.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let elapsed = op_start.elapsed();
                    if elapsed > std::time::Duration::from_millis(100) {
                        timing_print!("[process_op_table] Op '{}' took {:?}", name, elapsed);
                    }
                    if slowest_op.as_ref().map_or(true, |(_, d)| elapsed > *d) {
                        slowest_op = Some((name.clone(), elapsed));
                    }
                }
                // MEMOIZATION: Update defs so subsequent lookups get the normalized form
                defs.ops.insert(&scope_name, &name, Term::Nf(nf.clone()));
                result.ops.insert(&scope_name, &name, Term::Nf(nf));
                op_count += 1;
            }
            Term::FunDef(fun) => {
                result.ops.insert(&scope_name, &name, Term::FunDef(fun.to_owned()));
            }
            // Lists and generators are STRUCTURAL, like FunDef — consumers
            // need the term itself, not its flattened result: `*gen` must
            // re-generate (seeded) per use, and named lists are indexed
            // (`list @ [1, 2]`). Memoizing them to Nf broke both ("Using
            // non-generator as generator" / list-index mocks). Pass through —
            // EXCEPT the render roots (`main`, `expect`), which downstream
            // code consumes as Nf.
            Term::Lop(lop) => {
                if name == "main" || name == "expect" {
                    let mut nf = NormalForm::init();
                    lop.apply_to_normal_form(&mut nf, &mut defs)?;
                    defs.ops.insert(&scope_name, &name, Term::Nf(nf.clone()));
                    result.ops.insert(&scope_name, &name, Term::Nf(nf));
                } else {
                    result.ops.insert(&scope_name, &name, Term::Lop(lop.to_owned()));
                }
            }
            Term::Gen(generator) => {
                if name == "main" || name == "expect" {
                    let mut nf = NormalForm::init();
                    generator.apply_to_normal_form(&mut nf, &mut defs)?;
                    defs.ops.insert(&scope_name, &name, Term::Nf(nf.clone()));
                    result.ops.insert(&scope_name, &name, Term::Nf(nf));
                } else {
                    result.ops.insert(&scope_name, &name, Term::Gen(generator.to_owned()));
                }
            }
        };
    }

    timing_print!("[process_op_table] Total: {:?} ({} ops, {} pre-normalized)", total_start.elapsed(), op_count, nf_count);
    #[cfg(not(target_arch = "wasm32"))]
    if let Some((name, duration)) = slowest_op {
        timing_print!("[process_op_table] Slowest op: '{}' took {:?}", name, duration);
    }

    result.ops.stems = defs.ops.stems.to_owned();
    // Carry the recordings registry forward and surface any unresolved
    // `Perform("name")` names collected during normalization so the host
    // (kintaro's DAW) can pre-arm tracks for them.
    result.recordings = defs.recordings.clone();
    result.pending_performs = defs.pending_performs.clone();

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
/// Strip kintaro's `warp` extensions so vanilla weresocool can parse a file
/// that uses them. Removes:
///   - top-level `warp NAME = { ... }` blocks (collects names along the way)
///   - inline `| warp { ... }` blocks within def bodies
///   - `| <name>` chain ops where `<name>` is one of the collected warp names
///
/// Replaces removed regions with spaces so byte offsets in any downstream
/// error messages still point to roughly the right place. Pure stripping —
/// the audio interpretation is unaffected because warps never produce sound.
pub fn strip_warp_extensions(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out: Vec<u8> = src.as_bytes().to_vec();

    // Pass 1: find `warp NAME = { ... }` blocks. Collect names + blank them out.
    let mut warp_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < bytes.len() {
        // Word-boundary check before `warp` keyword.
        let at_boundary = i == 0 || !is_socool_ident_byte(bytes[i - 1]);
        if at_boundary && i + 4 <= bytes.len() && &bytes[i..i + 4] == b"warp" {
            let after_kw = i + 4;
            if after_kw < bytes.len() && !is_socool_ident_byte(bytes[after_kw]) {
                // Eat whitespace, then read NAME.
                let mut j = after_kw;
                while j < bytes.len() && (bytes[j] as char).is_whitespace() { j += 1; }
                let name_start = j;
                while j < bytes.len() && is_socool_ident_byte(bytes[j]) { j += 1; }
                if j > name_start {
                    let name = std::str::from_utf8(&bytes[name_start..j]).unwrap().to_string();
                    // Optional blend-mode keyword between NAME and `=`
                    // (kintaro's `warp shadow multiply = { ... }` form).
                    let mut k = j;
                    while k < bytes.len() && (bytes[k] as char).is_whitespace() { k += 1; }
                    let mode_start = k;
                    let mut mode_end = k;
                    while mode_end < bytes.len() && is_socool_ident_byte(bytes[mode_end]) { mode_end += 1; }
                    if mode_end > mode_start {
                        let word = std::str::from_utf8(&bytes[mode_start..mode_end]).unwrap();
                        if matches!(word, "additive" | "add" | "over" | "multiply" | "mult" | "screen") {
                            k = mode_end;
                        }
                    }
                    // Eat whitespace, expect `=`, then `{`.
                    while k < bytes.len() && (bytes[k] as char).is_whitespace() { k += 1; }
                    if k < bytes.len() && bytes[k] == b'=' {
                        k += 1;
                        while k < bytes.len() && (bytes[k] as char).is_whitespace() { k += 1; }
                        if k < bytes.len() && bytes[k] == b'{' {
                            if let Some(close) = find_matching_brace(bytes, k) {
                                warp_names.insert(name);
                                blank_range(&mut out, i, close + 1);
                                i = close + 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }

    // Pass 2: find `| warp { ... }` inline blocks (anywhere in source).
    let bytes2 = out.clone();
    let mut i = 0;
    while i < bytes2.len() {
        if bytes2[i] == b'|' {
            let mut j = i + 1;
            while j < bytes2.len() && matches!(bytes2[j], b' ' | b'\t') { j += 1; }
            if j + 4 <= bytes2.len() && &bytes2[j..j + 4] == b"warp" {
                let after_kw = j + 4;
                if after_kw >= bytes2.len() || !is_socool_ident_byte(bytes2[after_kw]) {
                    let mut k = after_kw;
                    while k < bytes2.len() && matches!(bytes2[k], b' ' | b'\t') { k += 1; }
                    if k < bytes2.len() && bytes2[k] == b'{' {
                        if let Some(close) = find_matching_brace(&bytes2, k) {
                            blank_range(&mut out, i, close + 1);
                            i = close + 1;
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    // Pass 3: find `| <warpname>` chain refs and blank them.
    if !warp_names.is_empty() {
        let bytes3 = out.clone();
        let mut i = 0;
        while i < bytes3.len() {
            if bytes3[i] == b'|' {
                let mut j = i + 1;
                while j < bytes3.len() && matches!(bytes3[j], b' ' | b'\t') { j += 1; }
                let name_start = j;
                while j < bytes3.len() && is_socool_ident_byte(bytes3[j]) { j += 1; }
                if j > name_start {
                    let candidate = std::str::from_utf8(&bytes3[name_start..j]).unwrap();
                    if warp_names.contains(candidate) {
                        blank_range(&mut out, i, j);
                        i = j;
                        continue;
                    }
                }
            }
            i += 1;
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
}

/// Strip kintaro's `draw` extensions so vanilla weresocool can parse a file
/// that uses them. Same pattern as `strip_warp_extensions` above — find the
/// three syntactic forms (`draw NAME = { … }`, `| draw { … }`, `| <draw>`)
/// and blank them in-place, preserving newlines so error spans stay aligned.
pub fn strip_draw_extensions(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out: Vec<u8> = src.as_bytes().to_vec();

    // Pass 1: `draw NAME = { … }` top-level blocks.
    let mut draw_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < bytes.len() {
        let at_boundary = i == 0 || !is_socool_ident_byte(bytes[i - 1]);
        if at_boundary && i + 4 <= bytes.len() && &bytes[i..i + 4] == b"draw" {
            let after_kw = i + 4;
            if after_kw < bytes.len() && !is_socool_ident_byte(bytes[after_kw]) {
                let mut j = after_kw;
                while j < bytes.len() && (bytes[j] as char).is_whitespace() { j += 1; }
                let name_start = j;
                while j < bytes.len() && is_socool_ident_byte(bytes[j]) { j += 1; }
                if j > name_start {
                    let name = std::str::from_utf8(&bytes[name_start..j]).unwrap().to_string();
                    let mut k = j;
                    while k < bytes.len() && (bytes[k] as char).is_whitespace() { k += 1; }
                    if k < bytes.len() && bytes[k] == b'=' {
                        k += 1;
                        while k < bytes.len() && (bytes[k] as char).is_whitespace() { k += 1; }
                        if k < bytes.len() && bytes[k] == b'{' {
                            if let Some(close) = find_matching_brace(bytes, k) {
                                draw_names.insert(name);
                                blank_range(&mut out, i, close + 1);
                                i = close + 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }

    // Pass 2: `| draw { … }` inline blocks.
    let bytes2 = out.clone();
    let mut i = 0;
    while i < bytes2.len() {
        if bytes2[i] == b'|' {
            let mut j = i + 1;
            while j < bytes2.len() && matches!(bytes2[j], b' ' | b'\t') { j += 1; }
            if j + 4 <= bytes2.len() && &bytes2[j..j + 4] == b"draw" {
                let after_kw = j + 4;
                if after_kw >= bytes2.len() || !is_socool_ident_byte(bytes2[after_kw]) {
                    let mut k = after_kw;
                    while k < bytes2.len() && matches!(bytes2[k], b' ' | b'\t') { k += 1; }
                    if k < bytes2.len() && bytes2[k] == b'{' {
                        if let Some(close) = find_matching_brace(&bytes2, k) {
                            blank_range(&mut out, i, close + 1);
                            i = close + 1;
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    // Pass 3: `| draw <name>` named chain refs, AND bare `| <name>` refs
    // where <name> is a known draw def (the form compositions actually
    // write: `bd = { ... | circles }`). Vanilla weresocool must drop both
    // so the audio parses; kintaro re-derives the routing itself.
    if !draw_names.is_empty() {
        let bytes3 = out.clone();
        let mut i = 0;
        while i < bytes3.len() {
            if bytes3[i] == b'|' {
                let mut j = i + 1;
                while j < bytes3.len() && matches!(bytes3[j], b' ' | b'\t') { j += 1; }
                // Optional `draw` keyword before the name.
                let mut k = j;
                if k + 4 <= bytes3.len()
                    && &bytes3[k..k + 4] == b"draw"
                    && (k + 4 == bytes3.len() || !is_socool_ident_byte(bytes3[k + 4]))
                {
                    k += 4;
                    while k < bytes3.len() && matches!(bytes3[k], b' ' | b'\t') { k += 1; }
                }
                let name_start = k;
                while k < bytes3.len() && is_socool_ident_byte(bytes3[k]) { k += 1; }
                if k > name_start {
                    let candidate = std::str::from_utf8(&bytes3[name_start..k]).unwrap();
                    if draw_names.contains(candidate) {
                        blank_range(&mut out, i, k);
                        i = k;
                        continue;
                    }
                }
            }
            i += 1;
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
}

fn is_socool_ident_byte(b: u8) -> bool {
    (b as char).is_ascii_alphanumeric() || b == b'_'
}

fn find_matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open], b'{');
    let mut depth = 1i32;
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => { depth -= 1; if depth == 0 { return Some(i); } }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Replace a byte range with spaces (preserving newlines) so positions in
/// the rest of the source don't shift — error spans remain correct.
fn blank_range(out: &mut [u8], start: usize, end: usize) {
    for b in &mut out[start..end] {
        if *b != b'\n' && *b != b'\r' {
            *b = b' ';
        }
    }
}

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
    /// Import statements from the source file
    pub imports: Vec<String>,
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
                // Preserve import statements for formatting
                imports: imports_needed,
            })
        }
        Err(error) => {
            if !quiet {
                eprintln!("\n");
            }

            if let lalrpop_util::ParseError::User { error: err_str } = &error {
                // Drum preset error (format: "location:preset:message").
                // Must be checked before the color shape — both start with
                // a numeric location.
                if let Some((line, column, msg)) = extract_preset_error(err_str, &composition, &source_map) {
                    if !quiet {
                        weresocool_error::ErrorDisplay {
                            source: &composition,
                            line,
                            column,
                            label: &msg,
                            use_cyan: false,
                            ..Default::default()
                        }.display(false);
                    }
                    return Err(ParseError { message: msg, line, column }.into_error());
                }

                // Color error (format: "location:colorname").
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

                        // Display error. No source_name here — this path
                        // is reached from the formatter as well, where
                        // we don't have one and don't render anyway.
                        if !quiet {
                            weresocool_error::ErrorDisplay {
                                source: &composition,
                                line,
                                column,
                                label: &format!("Invalid color '{}'", bad_color),
                                use_cyan: false,
                                ..Default::default()
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

            // Extract everything we want to show (location + expected
            // list + actual token) BEFORE handing off — the old code
            // here used `error.map_location` (which consumes the error)
            // just to get the location, throwing the rest away. Now we
            // pattern-match by ref and keep all of it.
            let extracted = ExtractedParseError::from_lalrpop(&error, &processed_composition);
            let (line, column) = handle_parse_error(&extracted, &composition, &source_map, None, quiet);

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
    source_name: Option<String>,
) -> Result<ParsedComposition, Error> {
    let mut defs: Defs = if let Some(defs) = prev_defs {
        defs
    } else {
        Default::default()
    };

    let ws_start = timing_now!();
    let (imports_needed, composition) = handle_whitespace_and_imports(vec_string)?;
    timing_print!("[parse_file] handle_whitespace_and_imports: {:?}", ws_start.elapsed());

    // NOTE: warp/draw stripping no longer happens here. The kintaro-DSL
    // front end (`weresocool::interpretable::preprocess_for_audio`) is the
    // ONE place visual blocks are extracted — every host runs it before
    // parse_file (Interpretable::make does it automatically). The old
    // strip_warp_extensions/strip_draw_extensions byte-scanners remain
    // exported for external callers but are no longer part of parsing.

    // Process WGSL blocks - extract them and replace with IDs
    // This validates each WGSL block and fails fast on the first error
    // quiet=false to show errors during actual parsing
    let wgsl_start = timing_now!();
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
        let parsed_composition = parse_file(
            vec_string,
            Some(defs.clone()),
            working_path.clone(),
            Some(filepath.to_string()),
        )?;

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

    let parse_start = timing_now!();
    let init = socool::SoCoolParser::new().parse(&mut defs, &processed_composition);
    timing_print!("[parse_file] SoCoolParser::parse: {:?}", parse_start.elapsed());

    match init {
        Ok(init) => {
            let op_table_start = timing_now!();
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

            if let lalrpop_util::ParseError::User { error: err_str } = &error {
                // Drum preset error (format: "location:preset:message") —
                // checked before the color shape.
                if let Some((line, column, msg)) = extract_preset_error(err_str, &composition, &source_map) {
                    weresocool_error::ErrorDisplay {
                        source: &composition,
                        line,
                        column,
                        label: &msg,
                        use_cyan: false,
                        file: source_name.clone(),
                        ..Default::default()
                    }.display(false);
                    return Err(ParseError { message: msg, line, column }.into_error());
                }

                // Color error (format: "location:colorname").
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
                            file: source_name.clone(),
                            ..Default::default()
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

            // See the formatter site above: pattern-match the lalrpop
            // error by ref so the expected/actual-token info survives
            // into the display.
            let extracted = ExtractedParseError::from_lalrpop(&error, &processed_composition);
            let (line, column) = handle_parse_error(
                &extracted,
                &composition,
                &source_map,
                source_name.as_deref(),
                false,
            );

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

/// Validate a drum preset name at parse time. Generic over the lexer token
/// type so the lalrpop action's `?` can convert the error directly. Unknown
/// names get an error listing the available presets — much better UX than
/// a silent fallback (typos should never quietly change the sound).
pub fn validate_drum_preset<T>(
    location: usize,
    drum: &str,
    preset: &str,
    available: &[&str],
) -> Result<(), lalrpop_util::ParseError<usize, T, String>> {
    if available.contains(&preset) {
        Ok(())
    } else {
        // `location:preset:` marker — the error sites in this file key on
        // it to render a preset-specific message (the bare `location:msg`
        // shape is claimed by the color-error path).
        Err(lalrpop_util::ParseError::User {
            error: format!(
                "{}:preset:unknown {} preset `{}` — available: {}",
                location,
                drum,
                preset,
                available.join(", ")
            ),
        })
    }
}

/// Drum preset errors arrive as `location:preset:message` (see
/// `validate_drum_preset`). Returns `(line, column, message)` when the
/// User error is preset-shaped, mapping the location through the source
/// map exactly like the color-error path does.
fn extract_preset_error(
    err_str: &str,
    composition: &str,
    source_map: &SourceMap,
) -> Option<(usize, usize, String)> {
    let (loc_str, rest) = err_str.split_once(':')?;
    let msg = rest.strip_prefix("preset:")?;
    let loc = loc_str.parse::<usize>().ok()?;
    let start = source_map.to_original(loc);
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
    Some((line, column, msg.to_string()))
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
