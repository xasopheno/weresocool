//! Shared byte-scanning primitives for the visual-DSL preprocessors
//! (warp / draw / surface). Each `extract_*` pass strips `NAME = { … }` blocks
//! and `| ref` clauses from .socool source before the audio engine sees it;
//! these helpers are the common lexer-level machinery they all used to carry
//! as byte-identical private copies. Keeping one copy here is what stops the
//! three from drifting apart (e.g. the bare-vs-keyword ref handling).
//!
//! `reaches` (BFS over the def graph) stays per-DSL for now — it's coupled to
//! each module's local `DefInfo` shape.

use std::collections::HashSet;

/// Is `b` a valid first byte of an identifier?
pub(crate) fn is_ident_start(b: u8) -> bool {
    (b as char).is_ascii_alphabetic() || b == b'_'
}

/// Is `b` a valid non-first byte of an identifier?
pub(crate) fn is_ident_byte(b: u8) -> bool {
    (b as char).is_ascii_alphanumeric() || b == b'_'
}

/// Does the keyword `kw` sit at byte offset `i` with identifier word
/// boundaries on both sides (so `warp` matches but `warped` doesn't)?
pub(crate) fn matches_keyword(bytes: &[u8], i: usize, kw: &[u8]) -> bool {
    if i + kw.len() > bytes.len() { return false; }
    if &bytes[i..i + kw.len()] != kw { return false; }
    if i > 0 && is_ident_byte(bytes[i - 1]) { return false; }
    let after = i + kw.len();
    if after < bytes.len() && is_ident_byte(bytes[after]) { return false; }
    true
}

/// Advance past ASCII whitespace, returning the next non-space offset.
pub(crate) fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
    i
}

/// Given the offset of an opening `{`, return the offset of its matching `}`
/// (brace-balanced), or `None` if unbalanced.
pub(crate) fn find_matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
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

/// One reference clause found in a def body by [`scan_def_body`].
pub(crate) enum RefTag {
    /// `| name` (bare) or `| kw name` (keyword) → a named def reference.
    Named(String),
    /// `| kw { … }` → an inline anonymous block. `body` is the inner text
    /// (between the braces); `open`/`close` are the brace byte offsets in the
    /// scanned body, for callers that need a span (warp tweak slots).
    Inline { body: String, open: usize, close: usize },
}

/// Output of [`scan_def_body`]: the body with all DSL clauses elided, the
/// ordered tags found, and whether a `Color [...]` clause was present (the
/// brush-index gate).
pub(crate) struct ScanOut {
    pub stripped: String,
    pub tags: Vec<RefTag>,
    pub has_color: bool,
}

/// Scan one def body for visual-DSL clauses, eliding them from the returned
/// `stripped` source. Recognizes — uniformly across warp/draw/surface — three
/// reference forms (this is the single rule that ends the bare-vs-keyword
/// drift the three preprocessors used to have):
///   * bare      `| name`        (name ∈ `known`)
///   * keyword   `| kw name`     (name ∈ `known`)
///   * inline    `| kw { … }`    (only when `allow_inline`)
///
/// Everything else — including `Color`, which it counts but copies through —
/// passes verbatim. Line comments (`--`, `//`) are copied without clause
/// detection so a `| name` inside a comment is never mistaken for a ref.
pub(crate) fn scan_def_body(
    body: &str,
    keyword: &[u8],
    known: &HashSet<String>,
    allow_inline: bool,
) -> ScanOut {
    let bytes = body.as_bytes();
    let kw_len = keyword.len();
    let mut out = String::with_capacity(body.len());
    let mut tags: Vec<RefTag> = Vec::new();
    let mut has_color = false;
    let mut i = 0usize;

    while i < bytes.len() {
        // Copy line comments verbatim (no clause detection inside them).
        if (bytes[i] == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-')
            || (bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/')
        {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }

        if bytes[i] == b'|' {
            let j = skip_ws(bytes, i + 1);
            if matches_keyword(bytes, j, keyword) {
                let after = skip_ws(bytes, j + kw_len);
                // (a) inline `| kw { … }`
                if allow_inline && after < bytes.len() && bytes[after] == b'{' {
                    if let Some(close) = find_matching_brace(bytes, after) {
                        tags.push(RefTag::Inline {
                            body: body[after + 1..close].to_string(),
                            open: after,
                            close,
                        });
                        // Preserve newlines so downstream parse errors keep
                        // their line numbers.
                        for p in i..close + 1 {
                            if bytes[p] == b'\n' { out.push('\n'); }
                        }
                        i = close + 1;
                        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') { i += 1; }
                        continue;
                    }
                }
                // (b) keyword `| kw name`
                let mut k = after;
                while k < bytes.len() && is_ident_byte(bytes[k]) { k += 1; }
                if k > after && known.contains(&body[after..k]) {
                    tags.push(RefTag::Named(body[after..k].to_string()));
                    i = k;
                    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') { i += 1; }
                    continue;
                }
            } else {
                // (c) bare `| name`
                let mut k = j;
                while k < bytes.len() && is_ident_byte(bytes[k]) { k += 1; }
                if k > j && known.contains(&body[j..k]) {
                    tags.push(RefTag::Named(body[j..k].to_string()));
                    i = k;
                    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') { i += 1; }
                    continue;
                }
            }
        }

        if matches_keyword(bytes, i, b"Color") {
            has_color = true;
            out.push_str("Color");
            i += 5;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }

    ScanOut { stripped: out, tags, has_color }
}

/// Scan `body` for bare identifier tokens that name a known def (`def_names`),
/// skipping `--` and `//` line comments. Used to build the def-reference graph
/// for reachability routing.
pub(crate) fn collect_refs(body: &str, def_names: &HashSet<String>) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut refs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-' {
            while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            continue;
        }
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            continue;
        }
        if is_ident_start(bytes[i]) {
            let start = i;
            while i < bytes.len() && is_ident_byte(bytes[i]) { i += 1; }
            let tok = &body[start..i];
            if def_names.contains(tok) {
                refs.push(tok.to_string());
            }
            continue;
        }
        i += 1;
    }
    refs
}
