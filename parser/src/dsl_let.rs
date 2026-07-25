//! `let` bindings inside visual-DSL bodies — the missing complexity valve.
//!
//! A source→source transform run after `dsl_params` (so a binding may use a
//! template's parameters) and before every per-DSL preprocessor. A body may
//! open with a prelude of bindings:
//!
//! ```text
//! draw tower = {
//!     let h    = 0.27 + Note.y * 0.55     -- the tower's height
//!     let base = 0.356 - Note.y
//!     Point
//!     | Ya(base)
//!     | Lerp { to: Ya(0.74 * h), n: 40 }
//! }
//! ```
//!
//! Each name is replaced by its parenthesised expression everywhere later in
//! the body, so `0.74 * h` becomes `0.74 * (0.27 + Note.y * 0.55)`. The
//! grammars, ASTs and codegen never learn bindings exist — by the time they
//! run, the text is what it always was.
//!
//! Why this rather than a grammar-level `let`: the visual DSLs have three
//! separate grammars (warp/draw/surface) plus a wgsl one, and an expression
//! binding would have to be threaded through all of them and their evaluators.
//! Textual expansion buys the whole feature once, exactly as parameterized
//! defs did.
//!
//! Rules, and the reasons for them:
//!
//! * **Prelude only.** Bindings sit at the top of a body, before the pipeline.
//!   A binding in the middle of a chain would read as if it were sequenced
//!   with the ops around it, which it is not — it has no position in time.
//! * **Sequential.** A binding may use the ones above it, so intermediate
//!   arithmetic can be built up in steps.
//! * **lowercase only.** CASE LAW: uppercase names belong to the language
//!   (`Note`, `Time`, `Rand`), lowercase to the user. Refusing to bind an
//!   uppercase name makes shadowing a language atom impossible.
//! * **`Raw { … }` bodies are skipped.** Those are WGSL, which has its own
//!   `let` and its own identifiers; substituting there could silently corrupt
//!   a shader. Inside Raw you already have real bindings.

use crate::dsl_extract::{find_matching_brace, is_ident_byte, is_ident_start, matches_keyword, skip_ws};

/// Bodies that may carry a binding prelude. `wgsl` is absent on purpose: it
/// lives in its own grammar with a different statement shape.
const KEYWORDS: [&str; 4] = ["warp", "draw", "surface", "layer"];

/// Expand every `let` prelude in the source. Fast no-op when there are none.
pub fn expand(source: &str) -> Result<String, String> {
    if !source.contains("let ") {
        return Ok(source.to_string());
    }
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut copied = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // A def keyword at a token boundary, with a `{` close behind it.
        let at_boundary = i == 0 || !is_ident_byte(bytes[i - 1]);
        let kw = if at_boundary {
            KEYWORDS.iter().find(|kw| matches_keyword(bytes, i, kw.as_bytes()))
        } else {
            None
        };
        let Some(kw) = kw else {
            i += 1;
            continue;
        };
        let Some(brace) = header_brace(bytes, i + kw.len()) else {
            i += 1;
            continue;
        };
        let Some(body_close) = find_matching_brace(bytes, brace) else {
            i += 1;
            continue;
        };

        let body = &source[brace + 1..body_close];
        match rewrite_body(body)? {
            Some(rewritten) => {
                out.push_str(&source[copied..brace + 1]);
                out.push_str(&rewritten);
                copied = body_close;
            }
            None => {}
        }
        i = body_close + 1;
    }

    out.push_str(&source[copied..]);
    Ok(out)
}

/// The `{` opening a def body, scanning from just past the keyword. Headers
/// are short and single-line (`name`, `name over =`, or nothing at all for an
/// anonymous `layer`), so a `{` further off than that is somebody else's.
fn header_brace(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    let limit = (from + 160).min(bytes.len());
    let mut newlines = 0;
    while i < limit {
        match bytes[i] {
            b'{' => return Some(i),
            b'\n' => {
                newlines += 1;
                if newlines > 1 {
                    return None;
                }
            }
            // another body closed, or a nested call — not a def header
            b'}' => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Strip a body's binding prelude and substitute the bindings through the
/// rest of it. `None` when the body opens with no binding (the overwhelming
/// case — leave those bytes untouched).
fn rewrite_body(body: &str) -> Result<Option<String>, String> {
    let bytes = body.as_bytes();
    let mut binds: Vec<(String, String)> = Vec::new();
    let mut pos = 0usize;

    loop {
        let scan = skip_blank_and_comments(bytes, pos);
        if !matches_keyword(bytes, scan, b"let") {
            pos = if binds.is_empty() { pos } else { scan };
            break;
        }
        let name_start = skip_ws(bytes, scan + 3);
        if name_start >= bytes.len() || !is_ident_start(bytes[name_start]) {
            return Err("`let` must be followed by a name".into());
        }
        let mut name_end = name_start + 1;
        while name_end < bytes.len() && is_ident_byte(bytes[name_end]) {
            name_end += 1;
        }
        let name = &body[name_start..name_end];
        if name.as_bytes()[0].is_ascii_uppercase() {
            return Err(format!(
                "`let {name}`: bindings must start lowercase — uppercase names belong to the language"
            ));
        }
        let eq = skip_ws(bytes, name_end);
        if eq >= bytes.len() || bytes[eq] != b'=' {
            return Err(format!("`let {name}` needs an `=`"));
        }
        let mut end = eq + 1;
        while end < bytes.len() && bytes[end] != b'\n' {
            end += 1;
        }
        let raw = body[eq + 1..end].trim();
        let expr = strip_trailing_comment(raw).trim();
        if expr.is_empty() {
            return Err(format!("`let {name}` has no expression"));
        }
        // Sequential: a binding sees the ones above it.
        let expr = substitute(expr, &binds);
        binds.push((name.to_string(), expr));
        pos = end;
    }

    if binds.is_empty() {
        return Ok(None);
    }
    Ok(Some(substitute(&body[pos..], &binds)))
}

/// Advance past whitespace and whole comment lines.
fn skip_blank_and_comments(bytes: &[u8], mut i: usize) -> usize {
    loop {
        i = skip_ws_nl(bytes, i);
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else {
            return i;
        }
    }
}

fn skip_ws_nl(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    i
}

/// Drop a trailing `--`/`//` comment from a binding's expression.
fn strip_trailing_comment(s: &str) -> &str {
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if (b[i] == b'-' && b[i + 1] == b'-') || (b[i] == b'/' && b[i + 1] == b'/') {
            return &s[..i];
        }
        i += 1;
    }
    s
}

/// Replace token-boundary occurrences of each bound name with `(expr)`.
/// Comments, named-argument keys (`amp:`) and `Raw { … }` bodies are left
/// exactly as they were.
fn substitute(text: &str, binds: &[(String, String)]) -> String {
    if binds.is_empty() {
        return text.to_string();
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Raw bodies are WGSL — hands off.
        if matches_keyword(bytes, i, b"Raw") {
            let after = skip_ws(bytes, i + 3);
            if after < bytes.len() && bytes[after] == b'{' {
                if let Some(close) = find_matching_brace(bytes, after) {
                    i = close + 1;
                    continue;
                }
            }
        }
        if is_ident_start(bytes[i]) && (i == 0 || !is_ident_byte(bytes[i - 1])) {
            let mut k = i + 1;
            while k < bytes.len() && is_ident_byte(bytes[k]) {
                k += 1;
            }
            // `name:` is a named-arg KEY, never a value.
            let is_key = k < bytes.len() && bytes[k] == b':';
            // `a.b` — only substitute a bare head, never a field access.
            let is_field = i > 0 && bytes[i - 1] == b'.';
            if !is_key && !is_field {
                if let Some((_, expr)) = binds.iter().find(|(n, _)| n == &text[i..k]) {
                    out.push_str(&text[last..i]);
                    out.push('(');
                    out.push_str(expr);
                    out.push(')');
                    last = k;
                }
            }
            i = k;
        } else {
            i += 1;
        }
    }
    out.push_str(&text[last..]);
    out
}

fn is_line_comment(bytes: &[u8], i: usize) -> bool {
    i + 1 < bytes.len()
        && ((bytes[i] == b'/' && bytes[i + 1] == b'/') || (bytes[i] == b'-' && bytes[i + 1] == b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_bindings_is_untouched() {
        let src = "draw a = {\n    Point | Ya(0.5)\n}\n";
        assert_eq!(expand(src).unwrap(), src);
    }

    #[test]
    fn binding_substitutes_parenthesised() {
        let src = "draw a = {\n    let h = 0.2 + Note.y\n    Point | Ya(h) | Sm(h * 2)\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Ya((0.2 + Note.y))"), "{out}");
        assert!(out.contains("Sm((0.2 + Note.y) * 2)"), "{out}");
        assert!(!out.contains("let h"), "prelude should be gone: {out}");
    }

    #[test]
    fn bindings_are_sequential() {
        let src = "warp w = {\n    let a = 2\n    let b = a * 3\n    Prev | Gain(b)\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Gain(((2) * 3))"), "{out}");
    }

    #[test]
    fn named_arg_keys_survive() {
        let src = "draw a = {\n    let n = 4\n    Point | Lerp { to: Ya(n), n: 12 }\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("n: 12"), "key must not be substituted: {out}");
        assert!(out.contains("Ya((4))"), "{out}");
    }

    #[test]
    fn raw_bodies_are_left_alone() {
        let src = "warp w = {\n    let g = 0.5\n    Prev | Gain(g)\n    | Raw { let g = 1.0; color = vec4<f32>(g, g, g, 1.0); }\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Gain((0.5))"), "{out}");
        assert!(out.contains("let g = 1.0; color = vec4<f32>(g, g, g, 1.0);"), "raw untouched: {out}");
    }

    #[test]
    fn uppercase_binding_is_rejected() {
        let src = "draw a = {\n    let Time = 3\n    Point\n}\n";
        assert!(expand(src).is_err());
    }

    #[test]
    fn field_access_is_not_substituted() {
        let src = "draw a = {\n    let y = 9\n    Point | Ya(Note.y + y)\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Note.y + (9)"), "{out}");
    }

    #[test]
    fn anonymous_layer_body_works() {
        let src = "layer {\n    let k = 0.25\n    Osc { freq: 3 } | Gain(k)\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Gain((0.25))"), "{out}");
    }

    #[test]
    fn trailing_comment_on_binding_is_dropped() {
        let src = "draw a = {\n    let h = 0.4   -- the height\n    Point | Ya(h)\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Ya((0.4))"), "{out}");
    }
}
