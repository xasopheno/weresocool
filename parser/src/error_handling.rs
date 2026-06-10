use crate::parser::SourceMap;
use strsim::jaro_winkler;
use weresocool_error::ErrorDisplay;

/// The user-visible operator/keyword vocabulary for SoCool, used by
/// the back-glance hint to fuzzy-match a stray identifier the parser
/// already accepted ("`m`") against a known op name ("`Fm`").
///
/// Sourced from `socool.lalrpop` — short multi-char ops (`Fm`, `Tm`,
/// …) and their full-word aliases (`Length`, `Gain`, …), plus the
/// chain keywords (`Sequence`, `Seq`, `Repeat`, …). Drop a new op
/// here when you add one to the grammar; the back-glance is best-effort
/// anyway, so an out-of-date entry just means no suggestion.
const SOCOOL_OP_VOCAB: &[&str] = &[
    // Short ops — these are the common typo targets.
    "Fm", "Tm", "Fa", "Ta", "Gm", "Lm", "Pm", "Pa",
    // Long-form aliases.
    "Gain", "Length", "PanM", "PanA",
    // Chain / list ops.
    "Sequence", "Seq", "Repeat", "Choose", "Random",
    // Audio + visual ops the parser also recognises in operand position.
    "AsIs", "FitGain", "FitLength", "Reverse", "Silence",
    "Sine", "Saw", "Noise",
];

/// Information extracted from a `lalrpop_util::ParseError` for display:
/// the byte position the parser tripped on, what it actually saw, and
/// what it was hoping to see.
///
/// We pull this OUT of the lalrpop error before `map_location` (which
/// consumes the error) instead of after — both because matching by ref
/// is cleaner and because the variants carry the actually-useful data
/// (`token`, `expected`) that was being silently discarded by the old
/// `error.map_location(|l| location.push(l))` pattern.
pub struct ExtractedParseError {
    pub byte_offset: usize,
    pub expected: Vec<String>,
    pub unexpected: Option<String>,
}

impl ExtractedParseError {
    /// `processed_source` is the (post-WGSL-stripping) string that was
    /// actually parsed — same one whose byte offsets the lalrpop error
    /// is using. For `InvalidToken` we use it to peek the offending
    /// text the lexer choked on, because the variant itself carries no
    /// token info — without this, the display would say "parse error
    /// at line 6 col 10" with no hint at what's wrong.
    pub fn from_lalrpop<T: std::fmt::Display, E>(
        error: &lalrpop_util::ParseError<usize, T, E>,
        processed_source: &str,
    ) -> Self {
        use lalrpop_util::ParseError::*;
        match error {
            InvalidToken { location } => Self {
                byte_offset: *location,
                expected: Vec::new(),
                unexpected: peek_token_at(processed_source, *location),
            },
            UnrecognizedEof { location, expected } => Self {
                byte_offset: *location,
                expected: expected.clone(),
                unexpected: Some("end of input".to_string()),
            },
            UnrecognizedToken {
                token: (start, t, _end),
                expected,
            } => Self {
                byte_offset: *start,
                expected: expected.clone(),
                unexpected: Some(format!("{}", t)),
            },
            ExtraToken { token: (start, t, _end) } => Self {
                byte_offset: *start,
                expected: Vec::new(),
                unexpected: Some(format!("{}", t)),
            },
            // User errors carry no location — fall back to 0; the
            // caller (e.g. the color-error path in parser.rs) usually
            // intercepts this variant before we get here.
            User { .. } => Self {
                byte_offset: 0,
                expected: Vec::new(),
                unexpected: None,
            },
        }
    }
}

/// Grab the token-shaped text at `byte_offset` in `source`. Used for
/// `InvalidToken` reporting where lalrpop tells us *where* the lexer
/// gave up but not *what* it was looking at. Returns `None` if we're
/// past EOF or on whitespace (the latter shouldn't happen with the
/// default lexer but we degrade gracefully).
fn peek_token_at(source: &str, byte_offset: usize) -> Option<String> {
    let tail = source.get(byte_offset..)?;
    // Identifier-like: gather the contiguous word the user typed.
    // Otherwise (punctuation, symbols), the first single char is
    // enough to communicate what tripped the lexer.
    let mut chars = tail.chars();
    let first = chars.next()?;
    if first.is_alphanumeric() || first == '_' {
        let word: String = std::iter::once(first)
            .chain(chars.take_while(|c| c.is_alphanumeric() || *c == '_'))
            .collect();
        Some(word)
    } else if first.is_whitespace() {
        None
    } else {
        Some(first.to_string())
    }
}

/// Resolve the error position into a (line, column) pair AND render
/// the unified `ErrorDisplay` view.
///
/// Both 1-based — matches what every editor shows. We walk by BYTE
/// offset (not char index) because the lalrpop location is a byte
/// offset; walking by char index breaks any source containing
/// multi-byte UTF-8 (e.g. `─` box-drawing chars in comment headers
/// take 3 bytes / 1 char each — the old char-index loop would
/// undershoot in char-space and overshoot by many lines).
pub fn handle_parse_error(
    extracted: &ExtractedParseError,
    original_composition: &str,
    source_map: &SourceMap,
    source_name: Option<&str>,
    quiet: bool,
) -> (usize, usize) {
    // Map processed → original byte offset.
    let start = source_map.to_original(extracted.byte_offset);

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

    // Back-glance hint: when the parser trips at a literal but the
    // token immediately before it in the source is a short identifier,
    // the user probably typo'd an op name (`m 9/8` for `Fm 9/8`). LALR
    // sees `m` as a valid Name and only errors at `9`, so without this
    // the recommendation list ends up being separators — useless.
    let note = build_back_glance_note(original_composition, start, &extracted);

    // Hand off to the colored byte-window display.
    ErrorDisplay {
        source: original_composition,
        line,
        column,
        label: "errors",
        use_cyan: false,
        expected: extracted.expected.clone(),
        unexpected: extracted.unexpected.clone(),
        file: source_name.map(|s| s.to_string()),
        note,
    }.display(quiet);

    (line, column)
}

/// Build the "parsed `X` as an operand at column N — did you mean `Y`?"
/// note. Returns `None` when the heuristic doesn't apply or doesn't
/// find a plausible match — we'd rather say nothing than mislead.
///
/// Gating:
///   - The unexpected token has to be a literal (digit or string),
///     otherwise the regular did-you-mean already handles it.
///   - The immediately-preceding token in the source has to be an
///     identifier of ≤4 chars. Longer than that and the user wasn't
///     making a typo of `Fm`/`Tm`/etc.
///   - The fuzzy match has to clear Jaro-Winkler ≥ 0.7 against the
///     known SoCool vocabulary; if nothing scores well we just say
///     "missing op prefix?" instead of guessing.
fn build_back_glance_note(
    source: &str,
    error_byte: usize,
    extracted: &ExtractedParseError,
) -> Option<String> {
    // Only when the unexpected token is a literal.
    let unexpected = extracted.unexpected.as_deref()?;
    let unexpected_is_literal = unexpected
        .chars()
        .next()
        .map(|c| c.is_ascii_digit() || c == '-' || c == '"')
        .unwrap_or(false);
    if !unexpected_is_literal { return None; }

    // Walk backward from the error byte through whitespace, then read
    // the preceding identifier (alphanumeric + `_`).
    let (preceding_text, preceding_start) = preceding_identifier(source, error_byte)?;
    if preceding_text.len() > 4 || preceding_text.is_empty() { return None; }
    if !preceding_text.chars().any(|c| c.is_alphabetic()) { return None; }

    // 1-based column of the preceding identifier.
    let preceding_column = column_at_byte(source, preceding_start);

    // Fuzzy-match against the known op vocab. Jaro-Winkler collapses
    // to 0 for very short strings (a 1-char `m` vs 2-char `Fm`), so we
    // ALSO consider any op that contains the preceding text as a
    // suffix — that's the "missing first letter" case the user usually
    // hits (`m` should be `Fm`/`Tm`/`Lm`/…).
    let lower = preceding_text.to_lowercase();
    let mut best: Option<(&str, f64)> = None;
    for &op in SOCOOL_OP_VOCAB {
        let score = jaro_winkler(&lower, &op.to_lowercase());
        if best.as_ref().map_or(true, |(_, s)| score > *s) {
            best = Some((op, score));
        }
    }
    let suffix_candidates: Vec<&str> = SOCOOL_OP_VOCAB
        .iter()
        .copied()
        .filter(|op| {
            // Strict suffix match, case-insensitive, only when:
            //   - the typed prefix is 1–2 chars
            //   - the candidate is at most 2 chars longer than the
            //     prefix (so `m` suggests `Fm`/`Tm`/…, not `PanM` or
            //     `Random` which just happen to end in `m`)
            preceding_text.len() <= 2
                && op.len() > preceding_text.len()
                && op.len() <= preceding_text.len() + 2
                && op.to_lowercase().ends_with(&lower)
        })
        .collect();

    let prefix = format!(
        "parsed `{}` as an operand at column {} —",
        preceding_text, preceding_column,
    );

    match best {
        Some((op, score)) if score >= 0.7 => Some(format!(
            "{} did you mean `{}`?",
            prefix, op,
        )),
        _ if !suffix_candidates.is_empty() => {
            let list = suffix_candidates
                .iter()
                .map(|op| format!("`{}`", op))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!(
                "{} did you mean one of {}?",
                prefix, list,
            ))
        }
        _ => Some(format!("{} missing op prefix?", prefix)),
    }
}

/// Scan backward from `byte_offset` past whitespace, then collect the
/// contiguous identifier (`[a-zA-Z_][a-zA-Z0-9_]*` shape). Returns
/// `(text, start_byte)` or `None` if no preceding identifier exists.
fn preceding_identifier(source: &str, byte_offset: usize) -> Option<(String, usize)> {
    let end = byte_offset.min(source.len());
    let head = &source[..end];

    // Skip trailing whitespace.
    let after_ws_end = head.trim_end().len();
    if after_ws_end == 0 { return None; }

    // Walk backward through the identifier body.
    let bytes = head.as_bytes();
    let mut start = after_ws_end;
    while start > 0 {
        let b = bytes[start - 1];
        let is_id = b == b'_'
            || b.is_ascii_alphanumeric();
        if !is_id { break; }
        start -= 1;
    }
    if start == after_ws_end { return None; }

    // First char must be alphabetic or `_` for this to be an identifier
    // shape rather than a number with letter suffix.
    let first = bytes[start];
    if !(first == b'_' || first.is_ascii_alphabetic()) { return None; }

    Some((head[start..after_ws_end].to_string(), start))
}

/// 1-based column of `byte_offset` in `source`. (Mirrors the loop in
/// `handle_parse_error`; pulled out so the back-glance can reuse it.)
fn column_at_byte(source: &str, byte_offset: usize) -> usize {
    let mut column: usize = 1;
    let mut byte: usize = 0;
    for c in source.chars() {
        if byte >= byte_offset { break; }
        byte += c.len_utf8();
        if c == '\n' { column = 1; } else { column += 1; }
    }
    column
}
