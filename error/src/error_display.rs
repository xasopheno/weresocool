use colored::*;
use std::io::Write;

/// Unified error display for all weresocool parser errors.
///
/// Renders a source window around the error position with:
///
///   - **Line-number gutter** (` 54 │ … `) on every shown line.
///   - **Window snapped to whole lines** so the top and bottom are
///     clean — never starts mid-line.
///   - **Yellow before / red after** color split at the exact byte the
///     parser tripped on. This is the only thing that's truthful even
///     when the line/column count is slightly off, so we lean on it.
///   - **Caret** (`^`) directly under the error column on the bad line.
///   - **Line + column** in the footer.
///
/// The byte-window split is preserved deliberately. A line-number-only
/// renderer (like ariadne) reports the wrong line and looks confidently
/// wrong when the upstream position is imprecise — which it can be,
/// because weresocool sources go through preprocessing and source-map
/// round-trips. The yellow/red split degrades gracefully: even if the
/// line is off, the user sees the actual source the parser was looking
/// at when it gave up.
///
/// The kintaro DSL parsers (warp/draw/surface) use `dsl_parse_error`
/// + ariadne instead — their positions ARE accurate (lalrpop gives
/// exact byte spans), so ariadne can be trusted.
pub struct ErrorDisplay<'a> {
    pub source: &'a str,
    /// 1-based line number of the error.
    pub line: usize,
    /// 1-based column number of the error.
    pub column: usize,
    /// Short category tag (`"errors"`, `"DSL error"`, …).
    pub label: &'a str,
    /// `true` = cyan/red palette (used by WGSL DSL); `false` = yellow/red.
    pub use_cyan: bool,
}

/// How many lines of context to show before/after the error line.
const LINES_BEFORE: usize = 3;
const LINES_AFTER: usize = 2;

impl<'a> ErrorDisplay<'a> {
    /// Render to stdout. `quiet` suppresses output.
    pub fn display(&self, quiet: bool) {
        if quiet { return; }

        // Signal TUI hosts FIRST so they can clear old content before
        // our window lands on stdout.
        println!("!");
        std::io::stdout().flush().ok();

        let error_pos = self.find_error_position();
        let len = self.source.len();

        // Snap window boundaries to whole lines.
        let window_start = back_n_lines(self.source, error_pos.min(len), LINES_BEFORE);
        let window_end = forward_n_lines(self.source, error_pos.min(len), LINES_AFTER);

        let first_line = count_newlines_before(self.source, window_start) + 1;

        // Gutter width = max digits across line numbers shown. We compute
        // the highest line number we'll print: first_line + newlines in
        // the window slice.
        let newlines_in_window =
            self.source[window_start..window_end].matches('\n').count();
        let last_line = first_line + newlines_in_window;
        let gutter_width = last_line.to_string().len();

        let primary_color = if self.use_cyan { Color::Cyan } else { Color::Yellow };

        let window = &self.source[window_start..window_end];
        let mut current_line = first_line;
        for line in window.split('\n') {
            let gutter = format!(" {:>w$} │ ", current_line, w = gutter_width);
            print!("{}", gutter.bright_black());

            if current_line == self.line {
                // Split on the error column.
                let col0 = self.column.saturating_sub(1);
                let split_byte = advance_chars(line, 0, col0).min(line.len());
                let before = &line[..split_byte];
                let after = &line[split_byte..];
                print!("{}", before.color(primary_color));
                println!("{}", after.red());

                // Caret line. Replicate the visual width of `before`
                // using spaces (tabs preserved as tabs so editors that
                // pad tabs consistently still line up).
                let caret_pad: String = before
                    .chars()
                    .map(|c| if c == '\t' { '\t' } else { ' ' })
                    .collect();
                let empty_gutter = format!(" {:>w$} │ ", "", w = gutter_width);
                println!(
                    "{}{}{}",
                    empty_gutter.bright_black(),
                    caret_pad,
                    "^".red().bold(),
                );
            } else if current_line < self.line {
                println!("{}", line.color(primary_color));
            } else {
                println!("{}", line.red());
            }

            current_line += 1;
        }

        // Footer — line + column, with the same color personality
        // ("working ← label → broken at line N, column M").
        println!();
        println!(
            "  {} ← {} → {} at line {}, column {}",
            "working".color(primary_color).underline(),
            self.label.bold(),
            "broken".red().underline(),
            self.line.to_string().red().bold(),
            self.column.to_string().red().bold(),
        );
        println!();

        std::io::stdout().flush().ok();
    }

    /// Map (line, column) → byte offset into `source`. 1-based inputs.
    fn find_error_position(&self) -> usize {
        let len = self.source.len();
        if self.line <= 1 {
            return advance_chars(self.source, 0, self.column.saturating_sub(1)).min(len);
        }
        let target = self.line - 1;
        let mut newlines_seen = 0usize;
        for (i, c) in self.source.char_indices() {
            if c == '\n' {
                newlines_seen += 1;
                if newlines_seen == target {
                    let line_start = i + 1;
                    return advance_chars(self.source, line_start, self.column.saturating_sub(1))
                        .min(len);
                }
            }
        }
        len
    }
}

/// Find the byte offset of the start of the line that's `n` lines BEFORE
/// the line containing `from`. Clamps to 0.
fn back_n_lines(source: &str, from: usize, n: usize) -> usize {
    let from = from.min(source.len());
    let mut newlines_found = 0usize;
    // Walk backward through char positions; remember the byte index of
    // each newline. Once we've passed `n + 1` newlines, the one BEFORE
    // that bounds the start of the line we want.
    let slice = &source[..from];
    for (i, c) in slice.char_indices().rev() {
        if c == '\n' {
            newlines_found += 1;
            if newlines_found > n {
                return i + 1; // byte after this newline = start of next line
            }
        }
    }
    0
}

/// Find the byte offset of the end of the line that's `n` lines AFTER
/// the line containing `from`. Clamps to `source.len()`.
fn forward_n_lines(source: &str, from: usize, n: usize) -> usize {
    let from = from.min(source.len());
    let mut newlines_found = 0usize;
    for (rel, c) in source[from..].char_indices() {
        if c == '\n' {
            newlines_found += 1;
            if newlines_found > n {
                return from + rel; // up to (not including) this newline
            }
        }
    }
    source.len()
}

/// Count newlines strictly before `byte_offset`. Used to map a byte
/// offset to a 0-based line number.
fn count_newlines_before(source: &str, byte_offset: usize) -> usize {
    let end = byte_offset.min(source.len());
    source[..end].chars().filter(|&c| c == '\n').count()
}

/// Advance `n` characters from `start` byte position in `source` and
/// return the resulting byte position. Stops at end of source. Used to
/// translate a 1-based column count into a byte offset on a line that
/// may contain multi-byte characters.
fn advance_chars(source: &str, start: usize, n: usize) -> usize {
    if start >= source.len() { return source.len(); }
    let mut iter = source[start..].char_indices();
    let mut last_end = start;
    for _ in 0..n {
        match iter.next() {
            Some((rel, c)) => last_end = start + rel + c.len_utf8(),
            None => break,
        }
    }
    last_end
}
