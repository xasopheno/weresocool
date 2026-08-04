//! Parameterized-def expansion (monomorphization). See
//! `docs/dsl-params-design.md`.
//!
//! A source→source transform run BEFORE the per-DSL preprocessors: a
//! parameterized template `warp glow(amp, decay) = { … }` plus a call
//! `| glow(0.01, 0.9)` become a specialized def `warp glow__<hash> = { … }`
//! (params textually substituted, parenthesized) and a bare reference
//! `| glow__<hash>`. The grammars/ASTs/codegen never learn params exist.
//!
//! Scope: `warp` / `draw` / `surface` templates (wgsl lives in weresocool and
//! is out of scope). Positional args only in v1.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::dsl_extract::{find_matching_brace, is_ident_byte, is_ident_start, matches_keyword, skip_ws};

/// Fixpoint backstop — bounds recursion / fan-out so a missing base case
/// reports an error instead of hanging.
const MAX_EXPANSIONS: usize = 4096;
const KEYWORDS: [&str; 3] = ["warp", "draw", "surface"];

#[derive(Debug, Clone)]
struct Template {
    keyword: String,
    params: Vec<String>,
    /// Text between the params `)` and the body `{` — reproduces ` <mode> = `.
    header_tail: String,
    body: String,
}

struct CallSite {
    name: String,
    args: Vec<String>,
    /// Byte span of `name(args)` in the working source.
    span: (usize, usize),
}

/// Expand all parameterized defs + their call sites. Returns the transformed
/// source (templates removed, specializations appended, calls rewritten to bare
/// refs). Fast no-op path when the source declares no parameterized defs.
pub fn expand(source: &str) -> Result<String, String> {
    let (mut src, templates) = extract_templates(source)?;
    if templates.is_empty() {
        return Ok(source.to_string());
    }
    let mut generated: HashSet<String> = HashSet::new();
    for _ in 0..MAX_EXPANSIONS {
        let Some(call) = find_call(&src, &templates) else {
            return Ok(src);
        };
        let t = &templates[&call.name];
        if call.args.len() != t.params.len() {
            return Err(format!(
                "parameterized `{}` expects {} arg(s), got {}",
                call.name,
                t.params.len(),
                call.args.len()
            ));
        }
        let spec = format!("{}__{}", call.name, hash_args(&call.name, &call.args));
        // Generate the specialization once per unique (name, args); its own body
        // may contain further calls, expanded on later iterations.
        if generated.insert(spec.clone()) {
            let body = substitute(&t.body, &t.params, &call.args);
            src.push_str(&format!("\n{} {}{}{{{}}}\n", t.keyword, spec, t.header_tail, body));
        }
        src.replace_range(call.span.0..call.span.1, &spec);
    }
    Err("parameterized-def expansion exceeded depth (recursion without a base case?)".into())
}

/// Pull every `<kw> NAME(params) <tail> = { body }` template out of the source,
/// returning the source with those headers removed and a name→template map.
fn extract_templates(source: &str) -> Result<(String, HashMap<String, Template>), String> {
    let bytes = source.as_bytes();
    let mut templates: HashMap<String, Template> = HashMap::new();
    let mut cut: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Never match keywords inside a line comment (a `warp Seq(…)` mention in
        // prose must not read as a def header).
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let kw = KEYWORDS.iter().find(|kw| matches_keyword(bytes, i, kw.as_bytes()));
        let Some(kw) = kw else {
            i += 1;
            continue;
        };
        let j = skip_ws(bytes, i + kw.len());
        if j >= bytes.len() || !is_ident_start(bytes[j]) {
            i += 1;
            continue;
        }
        let mut k = j + 1;
        while k < bytes.len() && is_ident_byte(bytes[k]) {
            k += 1;
        }
        let p = skip_ws(bytes, k);
        // Parameterized ONLY if a `(` follows the name. Plain `NAME = {…}` defs
        // fall through untouched.
        if p >= bytes.len() || bytes[p] != b'(' {
            i += 1;
            continue;
        }
        // From here, ANY shape mismatch means this wasn't a param-def header —
        // skip it (advance one byte), never error. Keeps false `kw ident(` hits
        // (in code or odd spots) from aborting the whole expansion.
        let Some(close) = find_matching_paren(bytes, p) else { i += 1; continue };
        let params: Vec<String> = split_args(&source[p + 1..close]);
        if params.iter().any(|pm| !is_valid_ident(pm)) {
            i += 1;
            continue;
        }
        // The header `) [mode] = {` must be single-line — bail on a newline or a
        // stray `{|(` before `=`.
        let mut e = close + 1;
        while e < bytes.len() && !matches!(bytes[e], b'=' | b'\n' | b'{' | b'|' | b'(') {
            e += 1;
        }
        if e >= bytes.len() || bytes[e] != b'=' {
            i += 1;
            continue;
        }
        let Some(brace) = find_byte_from(bytes, e + 1, b'{') else { i += 1; continue };
        // Only whitespace between `=` and `{`.
        if !source[e + 1..brace].trim().is_empty() {
            i += 1;
            continue;
        }
        let Some(body_close) = find_matching_brace(bytes, brace) else { i += 1; continue };
        let name = source[j..k].to_string();
        let tmpl = Template {
            keyword: kw.to_string(),
            params,
            header_tail: source[close + 1..brace].to_string(),
            body: source[brace + 1..body_close].to_string(),
        };
        if templates.insert(name.clone(), tmpl).is_some() {
            return Err(format!("duplicate parameterized def `{name}`"));
        }
        cut.push((i, body_close + 1));
        i = body_close + 1;
    }

    let mut out = String::with_capacity(source.len());
    let mut cur = 0;
    for (s, e) in &cut {
        out.push_str(&source[cur..*s]);
        cur = *e;
    }
    out.push_str(&source[cur..]);
    Ok((out, templates))
}

/// The first `NAME(args)` call to a known template in `src`, or `None`.
fn find_call(src: &str, templates: &HashMap<String, Template>) -> Option<CallSite> {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if is_ident_start(bytes[i]) && (i == 0 || !is_ident_byte(bytes[i - 1])) {
            let mut k = i + 1;
            while k < bytes.len() && is_ident_byte(bytes[k]) {
                k += 1;
            }
            let name = &src[i..k];
            if templates.contains_key(name) {
                // A CALL IS `name(` WITH NO GAP.
                //
                // This used to `skip_ws` between the name and the paren, and
                // that is not a stylistic nicety — it collides with the shape
                // `VERB channel (expr)`, which the substance verbs use
                // constantly:
                //
                //     | Decay pigment (1.0 - (lift))
                //
                // `pigment` is a state channel there AND the name of a warp
                // template in the medium library, so the whole line read as a
                // one-argument call to a two-argument template and the piece
                // died with "parameterized `pigment` expects 2 arg(s), got 1".
                // The library's own body does this, and so does
                // `jdbeck_orchestra`. Every call in the corpus is written
                // tight, which is also how every other language spells one.
                let p = k;
                if p < bytes.len() && bytes[p] == b'(' {
                    if let Some(close) = find_matching_paren(bytes, p) {
                        return Some(CallSite {
                            name: name.to_string(),
                            args: split_args(&src[p + 1..close]),
                            span: (i, close + 1),
                        });
                    }
                }
            }
            i = k;
        } else {
            i += 1;
        }
    }
    None
}

/// Replace every token-boundary occurrence of each param with `(arg)`. Line
/// comments are copied verbatim (no substitution inside them).
fn substitute(body: &str, params: &[String], args: &[String]) -> String {
    let map: HashMap<&str, &str> = params
        .iter()
        .map(String::as_str)
        .zip(args.iter().map(String::as_str))
        .collect();
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut last = 0; // start of not-yet-copied region
    let mut i = 0;
    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if is_ident_start(bytes[i]) && (i == 0 || !is_ident_byte(bytes[i - 1])) {
            let mut k = i + 1;
            while k < bytes.len() && is_ident_byte(bytes[k]) {
                k += 1;
            }
            // An ident directly followed by `:` is a named-arg KEY (`amp:`), not
            // a value — never substitute it, or `Bloom(strength: strength)` with
            // `strength` a param would corrupt the key.
            let is_named_key = k < bytes.len() && bytes[k] == b':';
            if !is_named_key {
                if let Some(arg) = map.get(&body[i..k]) {
                    out.push_str(&body[last..i]);
                    out.push('(');
                    out.push_str(arg);
                    out.push(')');
                    last = k;
                }
            }
            i = k;
        } else {
            i += 1;
        }
    }
    out.push_str(&body[last..]);
    out
}

fn is_line_comment(bytes: &[u8], i: usize) -> bool {
    i + 1 < bytes.len()
        && ((bytes[i] == b'/' && bytes[i + 1] == b'/') || (bytes[i] == b'-' && bytes[i + 1] == b'-'))
}

/// Split on top-level commas (paren/bracket/brace-depth aware). `""` → `[]`.
fn split_args(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.is_empty() {
        return Vec::new();
    }
    let bytes = s.as_bytes();
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (idx, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                args.push(s[start..idx].trim().to_string());
                start = idx + 1;
            }
            _ => {}
        }
    }
    args.push(s[start..].trim().to_string());
    args
}

fn find_matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open], b'(');
    let mut depth = 1i32;
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn find_byte_from(bytes: &[u8], from: usize, target: u8) -> Option<usize> {
    (from..bytes.len()).find(|&i| bytes[i] == target)
}

fn is_valid_ident(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && is_ident_start(b[0]) && b.iter().all(|&c| is_ident_byte(c))
}

/// Deterministic (fixed-seed `DefaultHasher`) so specialization names are stable
/// across runs — the routing golden depends on it.
fn hash_args(name: &str, args: &[String]) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut h);
    for a in args {
        a.hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_params_is_unchanged() {
        let src = "warp plain = { Prev | Decay 0.9 }\nmain = { plain }";
        assert_eq!(expand(src).unwrap(), src);
    }

    #[test]
    fn expands_warp_call() {
        let src = "warp glow(amp, decay) = { Prev | CurlFlow(amp) | Decay decay }\n\
                   main = { Prev | glow(0.01, 0.9) }";
        let out = expand(src).unwrap();
        // template header removed
        assert!(!out.contains("glow(amp"));
        // specialization generated with args substituted (parenthesized)
        assert!(out.contains("warp glow__"), "specialized def: {out}");
        assert!(out.contains("CurlFlow((0.01))"), "arg substituted: {out}");
        assert!(out.contains("Decay (0.9)"), "arg substituted: {out}");
        // call rewritten to a bare ref (no parens)
        assert!(!out.contains("glow(0.01"), "call rewritten: {out}");
    }

    #[test]
    fn same_args_dedupe_to_one_def() {
        let src = "draw s(n) = { Point | Spawn(n, Rz(1/4)) }\n\
                   a = { s(8) }\nb = { s(8) }\nc = { s(6) }";
        let out = expand(src).unwrap();
        // Two distinct specializations: s(8) shared by a & b, s(6) for c.
        assert_eq!(out.matches("draw s__").count(), 2, "2 specialized defs: {out}");
    }

    #[test]
    fn nested_param_call_expands() {
        // A template body that itself calls another template.
        let src = "draw dot(r) = { Point | Sm(r) }\n\
                   draw ring(r) = { dot(r) | Spawn(6, Rz(1/6)) }\n\
                   main = { ring(0.3) }";
        let out = expand(src).unwrap();
        assert!(out.contains("draw ring__"), "ring specialized: {out}");
        assert!(out.contains("draw dot__"), "nested dot specialized: {out}");
        // the inner `dot(r)` call is rewritten to a bare ref, not left as a call
        assert!(!out.contains("dot(("), "nested call rewritten: {out}");
        assert!(out.contains("0.3"), "value threaded through both levels: {out}");
    }

    #[test]
    fn named_arg_key_not_substituted() {
        // `amp` is both a named-arg KEY and a VALUE — only the value expands.
        let src = "warp g(amp) = { Prev | CurlFlow { amp: amp, freq: 6 } }\nmain = { g(0.01) }";
        let out = expand(src).unwrap();
        assert!(out.contains("CurlFlow { amp: (0.01), freq: 6 }"), "key kept, value expanded: {out}");
    }

    #[test]
    fn arity_mismatch_errors() {
        let src = "warp g(a, b) = { Prev }\nmain = { g(1) }";
        assert!(expand(src).is_err());
    }

    // The real Phase-0 chain: a parameterized def lives in an imported lib, and
    // the importer calls it. `dsl_imports::resolve` runs first, then `expand`
    // (mirrors `socool_processor`). This is the integration seam of the two new
    // source transforms.
    #[test]
    fn imports_then_params_compose() {
        let dir = std::env::temp_dir().join("kintaro_params_imports_seam");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("lib.socool"),
            "warp glow(amp) = { Prev | Decay amp }\n",
        )
        .unwrap();
        let main = "use \"lib\"\nmain = { Prev | glow(0.9) }";

        // 1. imports inline the lib (template comes along), 2. params expand it.
        let inlined = crate::dsl_imports::resolve(main, &dir).unwrap();
        assert!(inlined.contains("warp glow(amp)"), "template inlined: {inlined}");
        let out = expand(&inlined).unwrap();

        assert!(out.contains("warp glow__"), "specialized: {out}");
        assert!(out.contains("Decay (0.9)"), "arg substituted: {out}");
        assert!(!out.contains("glow(0.9)"), "call rewritten: {out}");
        assert!(!out.contains("use \""), "import resolved: {out}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
