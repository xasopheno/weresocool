//! `dsl_parse_error` — one parse-error type, one rustc-style display
//! path, for every kintaro DSL.
//!
//! ## Why
//!
//! kintaro has three internal DSLs — `warp`, `draw`, `surface_dsl` —
//! each with its own parser. Until this crate they all had a tiny
//! `ParseError(String)` that captured neither position nor source, so
//! the user got "expected ;, got Ident("foo")" with no context.
//!
//! Now every parser produces a `DslParseError` carrying:
//!
//!   * The original `source` text (we own a copy so the error is
//!     self-contained after creation).
//!   * A `byte_span` — start..end of the bad bit (zero-width if we
//!     only know the location).
//!   * The `label` of the DSL ("warp" / "draw" / "surface").
//!   * The error `message`.
//!   * Optional `expected` — the list of tokens the parser was
//!     prepared to see at that position (lalrpop gives us this for
//!     free; hand parsers can opt in).
//!   * Optional `help` — an actionable suggestion ("did you mean
//!     `Blur 1.5`?").
//!
//! `.display(quiet)` renders the error rustc-style via
//! [`ariadne`](https://github.com/zesterer/ariadne): source name in a
//! header, line numbers in a gutter, a colored caret under the bad
//! span, "expected" as the label message, and the help note at the
//! bottom. One place to change the rendering — every DSL benefits.

use ariadne::{Color, Label, Report, ReportKind};
use std::ops::Range;

/// A unified parse error across all kintaro DSLs.
#[derive(Clone)]
pub struct DslParseError {
    /// Short free-form message about what went wrong. Becomes the
    /// report's title line ("error[warp]: <message>").
    pub message: String,

    /// Half-open byte range `[start, end)` of the bad source region.
    /// `None` means the parser didn't know where; we fall back to the
    /// start of the source. A zero-width span (`start == end`) draws
    /// a single caret; a real span underlines the range.
    pub byte_span: Option<Range<usize>>,

    /// Which DSL produced the error. Becomes the bracketed code in
    /// the report header: `error[warp]`. Suggested values:
    /// `"warp"`, `"draw"`, `"surface"`.
    pub label: &'static str,

    /// The full source text the parser was looking at. We own a copy
    /// so display can render context without the caller having to
    /// thread the source through every call site.
    pub source: Option<String>,

    /// Logical name of the source, used in the report header
    /// ("<source_name>:line:col"). Defaults to `"<input>"` if unset.
    pub source_name: Option<String>,

    /// Tokens the parser was expecting at the failure point. lalrpop
    /// populates this from its grammar; hand parsers can call
    /// `with_expected` to add it. Empty list = "we didn't track."
    pub expected: Vec<String>,

    /// Optional actionable suggestion, rendered as the "Help:" line
    /// at the bottom of the report. Use for things like "did you
    /// mean `Blur`?" or "available draws: Mirror, Echo, …".
    pub help: Option<String>,
}

impl DslParseError {
    /// Build an error with no position yet. Useful at the top of a
    /// recursive-descent parser where you build the error first and
    /// add the source / span later.
    pub fn new(label: &'static str, message: impl Into<String>) -> Self {
        Self {
            label,
            message: message.into(),
            byte_span: None,
            source: None,
            source_name: None,
            expected: Vec::new(),
            help: None,
        }
    }

    /// Convenience constructor: error at a single byte offset
    /// (zero-width span). The most common case for tokenizers.
    pub fn at_byte(label: &'static str, byte_offset: usize, message: impl Into<String>) -> Self {
        Self {
            label,
            message: message.into(),
            byte_span: Some(byte_offset..byte_offset),
            source: None,
            source_name: None,
            expected: Vec::new(),
            help: None,
        }
    }

    /// Builder: attach (or replace) a zero-width position.
    pub fn at(mut self, byte_offset: usize) -> Self {
        self.byte_span = Some(byte_offset..byte_offset);
        self
    }

    /// Builder: attach a real source span. Use when you have a token
    /// with known start and end positions (lalrpop gives us this).
    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.byte_span = Some(span);
        self
    }

    /// Builder: attach (or replace) the source text.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Builder: attach a name for the source. Shows up in the report
    /// header (`<source_name>:line:col`).
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// Builder: list of tokens the parser was expecting at the error
    /// point. Rendered as the caret label.
    pub fn with_expected(mut self, expected: Vec<String>) -> Self {
        self.expected = expected;
        self
    }

    /// Builder: actionable hint shown at the bottom of the report.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Convert from a `lalrpop_util::ParseError`, preserving the full
    /// span (when available) and the "expected" set the grammar
    /// computed. The `source` is stored so `.display()` can render
    /// context later.
    pub fn from_lalrpop<T, E>(
        label: &'static str,
        err: lalrpop_util::ParseError<usize, T, E>,
        source: &str,
    ) -> Self
    where
        T: std::fmt::Display,
        E: std::fmt::Display,
    {
        use lalrpop_util::ParseError::*;
        let mut out = Self::new(label, "");
        out.source = Some(source.to_string());

        match err {
            InvalidToken { location } => {
                out.byte_span = Some(location..location);
                out.message = "invalid token".to_string();
            }
            UnrecognizedEof { location, expected } => {
                out.byte_span = Some(location..location);
                out.message = "unexpected end of input".to_string();
                out.expected = expected;
            }
            UnrecognizedToken { token, expected } => {
                out.byte_span = Some(token.0..token.2);
                out.message = format!("unexpected token `{}`", token.1);
                out.expected = expected;
            }
            ExtraToken { token } => {
                out.byte_span = Some(token.0..token.2);
                out.message = format!("extra token `{}` after end of valid input", token.1);
            }
            User { error } => {
                out.byte_span = Some(0..0);
                out.message = format!("{}", error);
            }
        }
        out
    }

    /// Render the error rustc-style via ariadne: header with
    /// source-name and DSL code, source snippet with line numbers,
    /// colored caret under the bad span, "expected" label, optional
    /// help note. Writes to stderr.
    ///
    /// `quiet` suppresses output entirely.
    ///
    /// If we have no source attached, falls back to the compact
    /// `Display` form.
    pub fn display(&self, quiet: bool) {
        if quiet { return; }
        let Some(source) = self.source.as_deref() else {
            eprintln!("{}", self);
            return;
        };

        let source_id = self
            .source_name
            .clone()
            .unwrap_or_else(|| "<input>".to_string());
        let span = self.byte_span.clone().unwrap_or(0..0);

        // ariadne's `Report::build` takes a `Span` whose `SourceId` we
        // get to choose. We use `(String, Range)` so the report header
        // shows the source name; the cache below ties that name to
        // the actual source text.
        let primary = (source_id.clone(), span.clone());

        let mut report = Report::build(ReportKind::Error, primary.clone())
            .with_code(self.label)
            .with_message(&self.message);

        // The label under the source: prefer the "expected one of"
        // hint (most useful when we have it), else echo the message.
        let label_msg = if !self.expected.is_empty() {
            if self.expected.len() == 1 {
                format!("expected {}", self.expected[0])
            } else {
                let nicely = self.expected.join(", ");
                format!("expected one of: {}", nicely)
            }
        } else {
            self.message.clone()
        };

        report = report.with_label(
            Label::new(primary.clone())
                .with_message(label_msg)
                .with_color(Color::Red),
        );

        if let Some(help) = &self.help {
            report = report.with_help(help);
        }

        // eprint takes a cache; the easiest way to build one for a
        // single named source is the `sources` helper.
        if let Err(e) = report
            .finish()
            .eprint(ariadne::sources([(source_id, source.to_string())]))
        {
            // ariadne shouldn't fail for in-memory sources, but
            // fall back to the compact form if it ever does.
            eprintln!("(ariadne render failed: {:?}; falling back)", e);
            eprintln!("{}", self);
        }
    }
}

impl std::fmt::Display for DslParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.byte_span, self.source.as_deref()) {
            (Some(s), Some(src)) => {
                let (line, col) = byte_to_line_col(src, s.start);
                write!(
                    f,
                    "{}: parse error at line {}, column {}: {}",
                    self.label,
                    line + 1,
                    col + 1,
                    self.message,
                )
            }
            _ => write!(f, "{}: parse error: {}", self.label, self.message),
        }
    }
}

impl std::fmt::Debug for DslParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for DslParseError {}

/// Convert a byte offset into a `(line, column)` pair (both 0-based).
/// Handles multi-byte chars correctly.
pub fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    let mut current = 0usize;
    for c in source.chars() {
        if current >= byte_offset { break; }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current += c.len_utf8();
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_to_line_col_basic() {
        let s = "abc\ndef\nghi";
        assert_eq!(byte_to_line_col(s, 0), (0, 0));
        assert_eq!(byte_to_line_col(s, 2), (0, 2));
        assert_eq!(byte_to_line_col(s, 4), (1, 0));
        assert_eq!(byte_to_line_col(s, 8), (2, 0));
        assert_eq!(byte_to_line_col(s, 99), (2, 3));
    }

    #[test]
    fn display_no_source_falls_back_to_compact() {
        let err = DslParseError::new("warp", "unexpected `,`");
        assert_eq!(format!("{}", err), "warp: parse error: unexpected `,`");
    }

    #[test]
    fn display_with_position_uses_line_col_in_compact_form() {
        let err = DslParseError::at_byte("draw", 6, "expected number")
            .with_source("Tau\n 2,\n");
        assert_eq!(
            format!("{}", err),
            "draw: parse error at line 2, column 3: expected number",
        );
    }

    #[test]
    fn builder_chain_carries_through() {
        let err = DslParseError::new("surface", "unknown source")
            .with_span(5..10)
            .with_source("Cube(1, 1)")
            .with_source_name("test.socool")
            .with_expected(vec!["Plane".to_string(), "Sphere".to_string()])
            .with_help("Cube isn't a surface source; try Plane or Sphere");
        assert_eq!(err.byte_span, Some(5..10));
        assert_eq!(err.expected.len(), 2);
        assert_eq!(err.help.as_deref(), Some("Cube isn't a surface source; try Plane or Sphere"));
        assert_eq!(err.source_name.as_deref(), Some("test.socool"));
    }
}
