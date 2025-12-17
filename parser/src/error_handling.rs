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

    // Calculate line and column from position
    let mut lines: usize = 0;
    let mut columns: usize = 0;
    for (n_c, c) in original_composition.chars().enumerate() {
        if n_c > start {
            break;
        }
        if c == '\n' {
            lines += 1;
            columns = 0;
        }
        columns += 1;
    }

    // Use unified error display
    ErrorDisplay {
        source: original_composition,
        line: lines,
        column: columns.saturating_sub(1),
        label: "errors",
        use_cyan: false,
    }.display(quiet);

    (lines, columns - 2)
}
