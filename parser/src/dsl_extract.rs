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

// ---------------------------------------------------------------------------
// THE ONE BLOCK SCANNER
// ---------------------------------------------------------------------------
//
// Every def kind in the language has the same shape — `KEYWORD name [tail] =
// { body }` — and until now every one of them carried its own byte-identical
// copy of the scan that finds it. Eight kinds, eight scanners, and they
// drifted: `warp` and `canvas` learned to skip comments (a commented-out def
// was still being compiled and applied), `light`, `text` and `draw` did not,
// so a commented-out `light` went on lighting the scene. That bug was written
// once and copied five times.
//
// One scanner. Each DSL supplies its keyword and its body parser.

/// One `KEYWORD name [tail] = { body }` block.
#[derive(Debug, Clone)]
pub struct DefBlock {
    pub name: String,
    /// Whatever sits between the name and the `=`. Almost always empty; it is
    /// where `warp shadow multiply = { … }` keeps its blend mode.
    pub tail: String,
    pub body: String,
    /// Byte range of the body WITHIN THE BRACES, in the original source. The
    /// warp promote-pass scopes its literal scan with this, so it must be the
    /// original offsets and not the blanked copy's.
    pub body_span: (usize, usize),
}

/// A block whose `{` never closed. Byte offset of the block start.
#[derive(Debug, Clone, Copy)]
pub struct UnbalancedBraces(pub usize);

/// Find every `KEYWORD name [tail] = { body }` in `source`.
///
/// Returns the source with each block replaced BYTE-FOR-BYTE by spaces
/// (newlines kept), plus the blocks in source order. Byte alignment is not a
/// nicety: the promote-pass walks the source text in parallel with the AST and
/// panics on desync.
///
/// Line comments are copied through WITHOUT being scanned. Prose talks about
/// code — the medium library documents its own idiom as `warp paint = { … }` —
/// and a comment-blind scan turns documentation into a def. It also means
/// commenting a def out is how you switch it off, which is the first thing
/// anyone tries.
pub fn scan_def_blocks(
    source: &str,
    keyword: &str,
) -> Result<(String, Vec<DefBlock>), UnbalancedBraces> {
    let bytes = source.as_bytes();
    let kw = keyword.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut blocks = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // `--` and `//` to end of line, verbatim, unscanned.
        if (bytes[i] == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-')
            || (bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/')
        {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(bytes[i]);
                i += 1;
            }
            continue;
        }
        if !matches_keyword(bytes, i, kw) {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        let block_start = i;
        let mut j = skip_ws(bytes, i + kw.len());
        let name_start = j;
        while j < bytes.len() && is_ident_byte(bytes[j]) {
            j += 1;
        }
        if j == name_start {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let name = source[name_start..j].to_string();

        // Anything between the name and `=` is the tail. Bail out on the
        // characters that mean this was never a def header, so an audio line
        // that merely starts with the keyword survives untouched.
        let tail_start = skip_ws(bytes, j);
        let mut k = tail_start;
        while k < bytes.len()
            && bytes[k] != b'='
            && bytes[k] != b'\n'
            && bytes[k] != b'{'
            && bytes[k] != b'|'
            && bytes[k] != b'('
        {
            k += 1;
        }
        if k >= bytes.len() || bytes[k] != b'=' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let tail = source[tail_start..k].trim().to_string();

        let brace = skip_ws(bytes, k + 1);
        if brace >= bytes.len() || bytes[brace] != b'{' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let body_start = brace + 1;
        let body_end = find_matching_brace(bytes, brace)
            .ok_or(UnbalancedBraces(block_start))?;

        blocks.push(DefBlock {
            name,
            tail,
            body: source[body_start..body_end].to_string(),
            body_span: (body_start, body_end),
        });

        let block_end = body_end + 1;
        for p in block_start..block_end {
            out.push(if bytes[p] == b'\n' { b'\n' } else { b' ' });
        }
        i = block_end;
    }

    Ok((
        String::from_utf8(out).expect("block scanner preserves UTF-8"),
        blocks,
    ))
}

#[cfg(test)]
mod scan_def_blocks_tests {
    use super::*;

    #[test]
    fn finds_a_def_and_blanks_it_byte_for_byte() {
        let src = "light sun = { gain: 1.0 }\nx = { Fm 1 }";
        let (stripped, defs) = scan_def_blocks(src, "light").unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "sun");
        assert_eq!(defs[0].body.trim(), "gain: 1.0");
        assert_eq!(stripped.len(), src.len());
        assert!(stripped.contains("x = { Fm 1 }"));
        assert!(!stripped.contains("sun"));
    }

    #[test]
    fn a_commented_def_is_prose() {
        // The bug this scanner exists to kill: a commented-out light went on
        // lighting the scene, and a commented-out draw was still compiled.
        let src = "-- light sun = { gain: 1.0 }\n-- draw ghost = { Point }\nx = { Fm 1 }";
        for kw in ["light", "draw"] {
            let (stripped, defs) = scan_def_blocks(src, kw).unwrap();
            assert!(defs.is_empty(), "{kw} found a def inside a comment");
            assert_eq!(stripped, src);
        }
    }

    #[test]
    fn captures_the_header_tail() {
        let (_, defs) = scan_def_blocks("warp shadow multiply = { Scene }", "warp").unwrap();
        assert_eq!(defs[0].name, "shadow");
        assert_eq!(defs[0].tail, "multiply");
    }

    #[test]
    fn a_def_merely_starting_with_the_keyword_is_left_alone() {
        for src in ["lighthouse = { Fm 1 }", "canvassing = { Fm 1 }"] {
            let (stripped, defs) = scan_def_blocks(src, "light").unwrap();
            assert!(defs.is_empty());
            assert_eq!(stripped, src);
        }
    }

    #[test]
    fn body_span_indexes_the_original_source() {
        let src = "zzz\nlight sun = { gain: 1.0 }";
        let (_, defs) = scan_def_blocks(src, "light").unwrap();
        let (a, b) = defs[0].body_span;
        assert_eq!(&src[a..b], " gain: 1.0 ");
    }

    #[test]
    fn nested_braces_close_at_the_right_one() {
        let src = "canvas c = { absorb: { density: 9 }, tooth: 0.2 }\nx = { Fm 1 }";
        let (_, defs) = scan_def_blocks(src, "canvas").unwrap();
        assert_eq!(defs[0].body.trim(), "absorb: { density: 9 }, tooth: 0.2");
    }

    #[test]
    fn unbalanced_braces_report_the_block_start() {
        let err = scan_def_blocks("light sun = { gain: 1.0", "light").unwrap_err();
        assert_eq!(err.0, 0);
    }
}
