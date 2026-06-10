use colored::*;
use std::io::Write;
use strsim::jaro_winkler;

/// Unified error display for all weresocool parser errors.
///
/// Renders a source window around the error position with:
///
///   - **Clickable file header** (`src/cull.socool:57:12`) when a path
///     is supplied — terminals like iTerm/VS Code make this jumpable.
///   - **Line-number gutter** (` 54 │ … `) on every shown line.
///   - **Window snapped to whole lines** so the top and bottom are
///     clean — never starts mid-line.
///   - **Yellow before / red after** color split at the exact byte the
///     parser tripped on. This is the only thing that's truthful even
///     when the line/column count is slightly off, so we lean on it.
///   - **Caret** (`^`) directly under the error column on the bad line.
///   - **Real message** (`Unexpected `;``) and **expected list**
///     (`expected one of: `,`, `}`, …`) in the footer when the lalrpop
///     parser hands them over.
///   - **Did-you-mean** suggestion via Jaro-Winkler when the unexpected
///     token is similar to one of the expected ones (`Decy` → `Decay`).
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
    /// Tokens the parser was expecting at this position. lalrpop strings
    /// are already quoted (e.g. `"\";\""`), and we unquote them for the
    /// "did you mean" comparison while preserving display formatting.
    /// Empty when the error site has no list (e.g. `InvalidToken`).
    pub expected: Vec<String>,
    /// The token that actually showed up at the error site, if known.
    /// Used to build "Unexpected `X`" and to drive did-you-mean lookups.
    pub unexpected: Option<String>,
    /// Path to the source file. When set, renders as a clickable
    /// `path:line:col` header that terminals can open in an editor.
    pub file: Option<String>,
    /// Optional extra hint line printed below the footer (renders as
    /// `note: …`). The intelligence to build this lives in the parser
    /// crate where the language vocabulary is known; the display just
    /// shows it. Currently used to surface back-glance suggestions
    /// ("parsed `m` as an operand at column 9 — did you mean `Fm`?")
    /// for the LALR-trips-one-token-late case.
    pub note: Option<String>,
}

impl<'a> Default for ErrorDisplay<'a> {
    fn default() -> Self {
        Self {
            source: "",
            line: 0,
            column: 0,
            label: "",
            use_cyan: false,
            expected: Vec::new(),
            unexpected: None,
            file: None,
            note: None,
        }
    }
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

        // Clickable file header. `path:line:col` is the format every
        // major terminal/editor recognises — iTerm, VS Code, JetBrains
        // all jump straight to the right spot when cmd-clicked.
        if let Some(file) = &self.file {
            println!(
                "{}",
                format!("  {}:{}:{}", file, self.line, self.column)
                    .bright_black()
                    .underline(),
            );
        }

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
        // ("working ← <message> → broken at line N, column M"). The
        // <message> is "Unexpected `X` (label)" when lalrpop told us
        // what token tripped it; otherwise the bare label.
        println!();
        let message = match &self.unexpected {
            Some(tok) => format!(
                "Unexpected {} ({})",
                format!("`{}`", tok).red().bold(),
                self.label,
            ),
            None => format!("{}", self.label.bold()),
        };
        println!(
            "  {} ← {} → {} at line {}, column {}",
            "working".color(primary_color).underline(),
            message,
            "broken".red().underline(),
            self.line.to_string().red().bold(),
            self.column.to_string().red().bold(),
        );

        // Expected-list line. lalrpop hands us its grammar-form
        // strings (`"Then"`, `r#"[0-9]+"#`, …); `pretty_lalrpop_terminal`
        // turns regex terminals into `<integer>` / `<number>` /
        // `<identifier>` placeholders and strips the outer quotes off
        // string terminals, then we de-dupe (multiple grammar rules
        // can map to the same friendly name).
        //
        // We deliberately suppress the line when every alternative is
        // pure separator/punctuation (`,`, `|`, `]`, …) AND we're not
        // at end of input. In operand-list grammars the parser usually
        // already consumed the offending operand, so the separators
        // are what *would* let it continue — they read like
        // recommendations but don't actually point at a fix.
        //
        // The EOF exception matters: when the source ends mid-expression
        // (`{ ... Tm 2` with no closing `}`), the expected list IS the
        // fix and we want to show `}` even though `}` is a separator.
        if !self.expected.is_empty() {
            let mut pretty: Vec<String> = self
                .expected
                .iter()
                .map(|t| pretty_lalrpop_terminal(t))
                .collect();
            pretty.sort();
            pretty.dedup();
            let any_actionable = pretty
                .iter()
                .any(|s| s.chars().any(|c| c.is_alphabetic()));
            let at_eof = self.unexpected.as_deref() == Some("end of input");
            if any_actionable || at_eof {
                let joined = pretty.join(", ");
                println!(
                    "  {} {}",
                    "expected one of:".bright_black(),
                    joined.color(primary_color),
                );
            }
        }

        // Caller-supplied note line. Used today for the back-glance
        // hint ("parsed `m` as an operand at column 9 — did you mean
        // `Fm`?") that catches LALR's one-token-late blind spot.
        if let Some(note) = &self.note {
            println!(
                "  {} {}",
                "note:".bright_blue().bold(),
                note,
            );
        }

        // Did-you-mean. Only when we have BOTH an actual unexpected
        // token and a non-empty expected list, AND something is close
        // enough to be a typo (Jaro-Winkler ≥ 0.75 is the usual
        // threshold for "obviously the same word").
        if let Some(hint) = self.did_you_mean() {
            println!(
                "  {} did you mean {}?",
                "hint:".bright_blue().bold(),
                format!("`{}`", hint).color(primary_color).bold(),
            );
        }

        println!();

        std::io::stdout().flush().ok();
    }

    /// Find the closest expected token to `unexpected` via Jaro-Winkler.
    /// Returns `None` if nothing is similar enough — we'd rather stay
    /// quiet than suggest something unrelated and confuse the user.
    fn did_you_mean(&self) -> Option<String> {
        let unexpected = self.unexpected.as_ref()?;
        // Strip the lalrpop quoting (`"\";\""` → `;`) so we compare
        // bare tokens. If the inner string is empty after unquoting
        // (which happens for punctuation that's the same as its quote
        // form), Jaro-Winkler is meaningless — skip.
        let cmp_unexpected = unquote_lalrpop_terminal(unexpected);
        if cmp_unexpected.is_empty() { return None; }
        // Only ID-like tokens (alphanumeric, at least 3 chars) benefit
        // from spelling suggestions. Suggesting "`;`" for a typo'd
        // identifier or vice versa is more noise than signal.
        if cmp_unexpected.len() < 3
            || !cmp_unexpected.chars().any(|c| c.is_alphabetic())
        {
            return None;
        }

        let mut best: Option<(String, f64)> = None;
        for expected in &self.expected {
            let cmp_expected = unquote_lalrpop_terminal(expected);
            if cmp_expected.len() < 3
                || !cmp_expected.chars().any(|c| c.is_alphabetic())
            {
                continue;
            }
            let score = jaro_winkler(
                &cmp_unexpected.to_lowercase(),
                &cmp_expected.to_lowercase(),
            );
            if best.as_ref().map_or(true, |(_, s)| score > *s) {
                best = Some((cmp_expected.to_string(), score));
            }
        }
        best.and_then(|(s, score)| if score >= 0.75 { Some(s) } else { None })
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

/// Strip the outer quotes lalrpop adds to terminal strings in its
/// "expected" list (`"Then"` → `Then`). Used for the did-you-mean
/// similarity check, where we want to compare bare token text. Regex
/// terminals (which start `r#"`) are passed through unchanged — their
/// shape is not meaningful for spelling suggestions, and they get
/// substituted to friendly names by `pretty_lalrpop_terminal` for
/// display purposes.
fn unquote_lalrpop_terminal(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Translate one lalrpop-style terminal into something a human can
/// read. lalrpop hands us:
///
///   - `"Then"`               → `Then`
///   - `r#"-?[0-9]+"#`        → `<integer>`
///   - `r#"-?...\\.\\d+..."#` → `<number>`
///   - `r#"[a-zA-Z_]..."#`    → `<identifier>`
///   - anything else          → passed through
///
/// The heuristics here are tuned for weresocool's `socool.lalrpop`
/// grammar specifically; adding new regex terminals there may want a
/// new branch here too.
fn pretty_lalrpop_terminal(s: &str) -> String {
    if let Some(inner) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
        return inner.to_string();
    }
    if let Some(body) = s.strip_prefix("r#\"").and_then(|x| x.strip_suffix("\"#")) {
        // Order matters: float patterns mention `[0-9]` too, so test
        // for the float-specific marker first.
        if body.contains("\\.") || body.contains("[eE]") {
            return "<number>".to_string();
        }
        if body.contains("[0-9]") {
            return "<integer>".to_string();
        }
        if body.contains("a-zA-Z") || body.contains("[_") {
            return "<identifier>".to_string();
        }
        if body.contains("\\\"") {
            return "<string>".to_string();
        }
        return "<token>".to_string();
    }
    s.to_string()
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
