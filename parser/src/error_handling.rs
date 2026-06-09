use std::sync::{Arc, Mutex};
use crate::parser::SourceMap;
use weresocool_error::ErrorDisplay;

pub fn handle_parse_error(
    location: Arc<Mutex<Vec<usize>>>,
    original_composition: &str,
    source_map: &SourceMap,
    quiet: bool,
) -> (usize, usize) {
    let cmp_len = &original_composition.len();
    let end = cmp_len;

    let loc = location.lock().unwrap();
    let arg_len = loc.len();

    // Get the error position in processed string, then map back to original
    // Default to 0 if no location available (e.g., for User errors)
    let processed_start = if arg_len > 0 { loc[0] } else { 0 };
    drop(loc);  // Release lock before calling to_original
    let start = source_map.to_original(processed_start);

    // Calculate (line, column) from the byte offset.
    //
    // Both 1-based — matches what every editor shows (line 1 is the
    // first line, column 1 is the first character on the line). The
    // previous impl returned 0-based lines and `columns - 2` (an
    // accumulated bandage that was wrong for the first column of a
    // line; it could underflow).
    //
    // IMPORTANT: we walk by BYTE offset, not char index. `start` is
    // a byte offset (it comes from lalrpop's `Loc` via `SourceMap`,
    // both of which deal in bytes). Walking by char index against a
    // byte cursor breaks any source with multi-byte UTF-8 chars
    // before the error — common, because composers use box-drawing
    // characters like `─` in comment headers (each `─` is 3 bytes /
    // 1 char). The old loop would undershoot in char-space and keep
    // counting newlines well past the real error, reporting (e.g.)
    // "line 134" for an error actually on line 57.
    let mut line: usize = 1;
    let mut column: usize = 1;
    let mut byte: usize = 0;
    for c in original_composition.chars() {
        if byte >= start { break; }
        byte += c.len_utf8();
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    // Hand off to the colored byte-window display.
    ErrorDisplay {
        source: original_composition,
        line,
        column,
        label: "errors",
        use_cyan: false,
    }.display(quiet);

    (line, column)
}
