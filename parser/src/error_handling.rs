use colored::*;
use std::cmp;
use std::sync::{Arc, Mutex};
use crate::parser::SourceMap;

pub fn handle_parse_error(
    location: Arc<Mutex<Vec<usize>>>,
    original_composition: &str,
    source_map: &SourceMap,
) -> (usize, usize) {
    let start_offset = 125;
    let end_offset = 50;
    let cmp_len = &original_composition.len();
    let end = cmp_len;

    let arg_len = location.lock().unwrap().len();
    if arg_len == 2 {
        let _end = location.lock().unwrap()[1];
    }

    // Get the error position in processed string, then map back to original
    let processed_start = location.lock().unwrap()[0];
    let start = source_map.to_original(processed_start);

    let feed_start = cmp::max(0, start as isize - start_offset) as usize;
    let mut feed_end = cmp::min(end + end_offset, *cmp_len);
    if feed_end - feed_start > 300 {
        feed_end = feed_start + 300
    }
    let mut lines = 0;
    let mut columns = 0;
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
    println!(
        "{}{}",
        &original_composition[feed_start..start].yellow(),
        &original_composition[start..feed_end].red(),
    );

    println!(
        "
            {}
            errors at line {}
            {}
            ",
        "working".yellow().underline(),
        lines.to_string().red().bold(),
        "broken".red().underline(),
    );

    (lines, columns - 2)
}
