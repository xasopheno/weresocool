//! Draw composition — naming a piece of a pipeline and reusing it.
//!
//! A source→source transform run after `dsl_params` and `dsl_let`. Inside a
//! `draw` body, a bare reference to another `draw` def splices that def's
//! text in, exactly as a concatenative language does: a definition IS a
//! sequence of operations, and its name stands for that sequence.
//!
//! ```text
//! draw thicken = { Spawn { n: 3, by: Xa(0.02) } }   -- a fragment of ops
//! draw shaft   = { Point | Lerp { to: Ya(0.3), n: 12 } }
//! draw tower   = { shaft | thicken }                -- composition
//! ```
//!
//! becomes `draw tower = { Point | Lerp { to: Ya(0.3), n: 12 } | Spawn { … } }`.
//!
//! Position decides meaning, and the composer decides what makes sense: a def
//! that opens with a generator (`Point`, `Path`) reads at the head of a
//! pipeline; a def that is only ops reads at a stage. Nothing needs to know
//! which is which — splicing text does the right thing either way.
//!
//! SCOPE — this is the important rule. References are expanded ONLY inside
//! `draw` bodies. A voice routing itself at a draw (`melody | tower`) is a
//! ROUTING reference, not composition, and must survive untouched; so must a
//! bare name inside a `warp` body, where it already means `Scene(name)` (the
//! image face). Composition is a draw-side idea only.

use std::collections::{HashMap, HashSet};

use crate::dsl_extract::{find_matching_brace, is_ident_byte, is_ident_start, matches_keyword, skip_ws};

/// Bounds fan-out so a cycle reports instead of hanging.
const MAX_PASSES: usize = 64;

/// Splice every draw-to-draw reference. No-op when a source has fewer than
/// two draw defs (nothing can refer to anything).
pub fn expand(source: &str) -> Result<String, String> {
    let mut src = source.to_string();
    for _ in 0..MAX_PASSES {
        let defs = collect_draw_defs(&src);
        if defs.len() < 2 {
            return Ok(src);
        }
        match expand_once(&src, &defs)? {
            Some(next) => src = next,
            None => return Ok(src),
        }
    }
    Err("draw composition exceeded depth — a def referring to itself?".into())
}

/// name → (body text, span of the whole `draw name = { … }`).
fn collect_draw_defs(source: &str) -> HashMap<String, String> {
    let bytes = source.as_bytes();
    let mut defs = HashMap::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let at_boundary = i == 0 || !is_ident_byte(bytes[i - 1]);
        if !(at_boundary && matches_keyword(bytes, i, b"draw")) {
            i += 1;
            continue;
        }
        let j = skip_ws(bytes, i + 4);
        if j >= bytes.len() || !is_ident_start(bytes[j]) {
            i += 1;
            continue;
        }
        let mut k = j + 1;
        while k < bytes.len() && is_ident_byte(bytes[k]) {
            k += 1;
        }
        let eq = skip_ws(bytes, k);
        if eq >= bytes.len() || bytes[eq] != b'=' {
            i += 1;
            continue;
        }
        let Some(brace) = find_byte_from(bytes, eq + 1, b'{') else { i += 1; continue };
        if !source[eq + 1..brace].trim().is_empty() {
            i += 1;
            continue;
        }
        let Some(close) = find_matching_brace(bytes, brace) else { i += 1; continue };
        defs.insert(source[j..k].to_string(), source[brace + 1..close].trim().to_string());
        i = close + 1;
    }
    defs
}

/// One substitution pass over every draw body. `None` when nothing changed.
fn expand_once(source: &str, defs: &HashMap<String, String>) -> Result<Option<String>, String> {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut copied = 0usize;
    let mut i = 0usize;
    let mut changed = false;

    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let at_boundary = i == 0 || !is_ident_byte(bytes[i - 1]);
        if !(at_boundary && matches_keyword(bytes, i, b"draw")) {
            i += 1;
            continue;
        }
        let j = skip_ws(bytes, i + 4);
        if j >= bytes.len() || !is_ident_start(bytes[j]) {
            i += 1;
            continue;
        }
        let mut k = j + 1;
        while k < bytes.len() && is_ident_byte(bytes[k]) {
            k += 1;
        }
        let self_name = &source[j..k];
        let eq = skip_ws(bytes, k);
        if eq >= bytes.len() || bytes[eq] != b'=' {
            i += 1;
            continue;
        }
        let Some(brace) = find_byte_from(bytes, eq + 1, b'{') else { i += 1; continue };
        let Some(close) = find_matching_brace(bytes, brace) else { i += 1; continue };

        let body = &source[brace + 1..close];
        let spliced = splice(body, defs, self_name)?;
        if let Some(spliced) = spliced {
            out.push_str(&source[copied..brace + 1]);
            out.push_str(&spliced);
            copied = close;
            changed = true;
        }
        i = close + 1;
    }

    if !changed {
        return Ok(None);
    }
    out.push_str(&source[copied..]);
    Ok(Some(out))
}

/// Replace bare draw names in one body with those defs' text. `None` when the
/// body holds no references.
fn splice(body: &str, defs: &HashMap<String, String>, self_name: &str) -> Result<Option<String>, String> {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut last = 0usize;
    let mut i = 0usize;
    let mut hit = false;

    while i < bytes.len() {
        if is_line_comment(bytes, i) {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Raw is WGSL — a name in there is not a draw reference.
        if matches_keyword(bytes, i, b"Raw") {
            let after = skip_ws(bytes, i + 3);
            if after < bytes.len() && bytes[after] == b'{' {
                if let Some(c) = find_matching_brace(bytes, after) {
                    i = c + 1;
                    continue;
                }
            }
        }
        if is_ident_start(bytes[i]) && (i == 0 || !is_ident_byte(bytes[i - 1])) {
            let mut k = i + 1;
            while k < bytes.len() && is_ident_byte(bytes[k]) {
                k += 1;
            }
            let name = &body[i..k];
            let is_key = k < bytes.len() && bytes[k] == b':';
            let is_field = i > 0 && bytes[i - 1] == b'.';
            // `name(` is a call, not a bare reference (params already ran).
            let is_call = {
                let p = skip_ws(bytes, k);
                p < bytes.len() && bytes[p] == b'('
            };
            if !is_key && !is_field && !is_call {
                if name == self_name && defs.contains_key(name) {
                    return Err(format!("draw `{name}` refers to itself"));
                }
                if let Some(text) = defs.get(name) {
                    out.push_str(&body[last..i]);
                    out.push_str(text);
                    last = k;
                    hit = true;
                }
            }
            i = k;
        } else {
            i += 1;
        }
    }

    if !hit {
        return Ok(None);
    }
    out.push_str(&body[last..]);
    Ok(Some(out))
}

fn find_byte_from(bytes: &[u8], from: usize, b: u8) -> Option<usize> {
    (from..bytes.len()).find(|&i| bytes[i] == b)
}

fn is_line_comment(bytes: &[u8], i: usize) -> bool {
    i + 1 < bytes.len()
        && ((bytes[i] == b'/' && bytes[i + 1] == b'/') || (bytes[i] == b'-' && bytes[i + 1] == b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_position_composes() {
        let src = "draw shaft = { Point | Lerp { to: Ya(0.3), n: 12 } }\ndraw tower = { shaft | Spawn { n: 3, by: Xa(0.02) } }\n";
        let out = expand(src).unwrap();
        assert!(out.contains("draw tower = { Point | Lerp { to: Ya(0.3), n: 12 } | Spawn"), "{out}");
    }

    #[test]
    fn stage_position_composes() {
        let src = "draw thicken = { Spawn { n: 3, by: Xa(0.02) } }\ndraw tower = { Point | thicken }\n";
        let out = expand(src).unwrap();
        assert!(out.contains("draw tower = { Point | Spawn { n: 3, by: Xa(0.02) } }"), "{out}");
    }

    #[test]
    fn chains_transitively() {
        let src = "draw a = { Point }\ndraw b = { a | Jitter 0.01 }\ndraw c = { b | Mirror(X) }\n";
        let out = expand(src).unwrap();
        assert!(out.contains("draw c = { Point | Jitter 0.01 | Mirror(X) }"), "{out}");
    }

    #[test]
    fn routing_references_are_untouched() {
        // `| tower` in an AUDIO def is routing, not composition.
        let src = "draw tower = { Point }\nmain = {\n    Seq [Fm 1]\n    | tower\n}\n";
        let out = expand(src).unwrap();
        assert!(out.contains("| tower\n"), "routing must survive: {out}");
    }

    #[test]
    fn warp_bodies_are_untouched() {
        // a bare name in a warp already means Scene(name)
        let src = "draw veil = { Point }\nwarp w = { Scene | Multiply(veil) }\n";
        let out = expand(src).unwrap();
        assert!(out.contains("Multiply(veil)"), "{out}");
    }

    #[test]
    fn self_reference_errors() {
        let src = "draw a = { Point }\ndraw loop = { loop | Jitter 0.01 }\n";
        assert!(expand(src).is_err());
    }

    #[test]
    fn single_def_is_noop() {
        let src = "draw a = { Point | Jitter 0.01 }\n";
        assert_eq!(expand(src).unwrap(), src);
    }
}
