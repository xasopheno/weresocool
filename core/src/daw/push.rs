//! "Push into the piece": promote a chosen take to a named recording and make
//! sure the armed voice references it via `Perform("name")`.
//!
//! Two arrival orders converge here (see plan):
//!   - arm-from-panel: the voice has no `Perform(...)` yet → we insert one.
//!   - pre-arm: the user already typed `Perform("name")` → no source edit.

use std::path::Path;

use super::store::DawStore;

/// Promote `take_id` to a recording named `name`, then ensure `armed_track`'s
/// definition in the source file references `Perform("name")`. Writing the
/// source triggers the file watcher's hot-reload, which re-transcribes and
/// resolves the new recording.
pub fn push_take(
    store: &mut DawStore,
    take_id: u64,
    name: &str,
    fps: usize,
    source_path: &Path,
    armed_track: &str,
) -> Result<(), String> {
    store
        .promote(take_id, name, fps)
        .map_err(|e| format!("promote: {e}"))?;

    let source = std::fs::read_to_string(source_path).map_err(|e| format!("read source: {e}"))?;
    if let Some(updated) = insert_perform_into_def(&source, armed_track, name) {
        std::fs::write(source_path, updated).map_err(|e| format!("write source: {e}"))?;
    }
    Ok(())
}

/// Insert `| Perform("name")` at the end of `track`'s definition body. Returns
/// the rewritten source, or `None` if the def can't be found or it already
/// references `Perform("name")` (idempotent — the pre-arm path).
pub fn insert_perform_into_def(source: &str, track: &str, name: &str) -> Option<String> {
    let (open, close) = find_def_braces(source, track)?;
    let body = &source[open + 1..close];
    let perform = format!("Perform(\"{}\")", name);
    if body.contains(&perform) {
        return None; // already present
    }

    // Insert before the closing brace, preserving the brace's leading layout.
    let insert_at = trim_end_index(source, open + 1, close);
    let mut out = String::with_capacity(source.len() + perform.len() + 8);
    out.push_str(&source[..insert_at]);
    out.push_str(" | ");
    out.push_str(&perform);
    out.push(' ');
    out.push_str(&source[insert_at..]);
    Some(out)
}

/// The body text (between the outer braces) of `name = { … }`, if present.
pub fn def_body<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let (open, close) = find_def_braces(source, name)?;
    Some(&source[open + 1..close])
}

/// Find the byte offsets of the `{` and matching `}` for `track = { ... }`.
/// Brace-counting handles nested `{}` (e.g. inline `wgsl { ... }`). Fragile
/// against braces inside strings/comments — acceptable for v1.
pub fn find_def_braces(source: &str, track: &str) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut search_from = 0;
    while let Some(rel) = source[search_from..].find(track) {
        let name_start = search_from + rel;
        let name_end = name_start + track.len();

        // Must be a standalone identifier (not a substring of a longer name).
        let prev_ok = name_start == 0
            || !is_ident_byte(bytes[name_start - 1]);
        let next = bytes.get(name_end).copied();
        let next_ok = next.map_or(false, |c| !is_ident_byte(c));
        if prev_ok && next_ok {
            // Expect `=` then `{` (allowing whitespace), and not a function def
            // `name(...) = {` — Perform-target voices are plain defs.
            let mut i = name_end;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if bytes.get(i) == Some(&b'=') {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                if bytes.get(i) == Some(&b'{') {
                    if let Some(close) = matching_brace(bytes, i) {
                        return Some((i, close));
                    }
                }
            }
        }
        search_from = name_end;
    }
    None
}

fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
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

/// Largest index in `[start, close)` such that everything after it up to
/// `close` is whitespace — i.e. insert position just after the last real token.
fn trim_end_index(source: &str, start: usize, close: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = close;
    while i > start && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    i
}

fn is_ident_byte(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric() || c == b'.'
}

// ── Source scanning ────────────────────────────────────────────────────────

/// The first voice whose def body references `Follow`.
pub fn follow_target(source: &str) -> Option<String> {
    voice_names(source)
        .into_iter()
        .find(|name| def_body(source, name).map_or(false, |b| b.contains("Follow")))
}

/// Top-level audio voice def names (`name = { … }`), excluding `main`, DSL
/// keyword blocks, and function defs.
pub fn voice_names(source: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in source.lines() {
        let t = line.trim_start();
        if t.starts_with("warp ")
            || t.starts_with("draw ")
            || t.starts_with("surface ")
            || t.starts_with("import ")
        {
            continue;
        }
        let Some(eq) = t.find('=') else { continue };
        let name = t[..eq].trim();
        if name.is_empty() || name == "main" {
            continue;
        }
        let is_ident = name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            && name
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_lowercase() || c == '_');
        if is_ident && !out.contains(&name.to_string()) {
            out.push(name.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_perform_into_simple_def() {
        let src = "{ f: 311, l: 1, g: 1, p: 0 }\nthing = { Fm 1/2 }\nmain = { thing }\n";
        let out = insert_perform_into_def(src, "thing", "sax").unwrap();
        assert!(out.contains("Fm 1/2 | Perform(\"sax\")"));
        // Other defs untouched.
        assert!(out.contains("main = { thing }"));
    }

    #[test]
    fn idempotent_when_already_present() {
        let src = "thing = { Fm 1/2 | Perform(\"sax\") }\n";
        assert!(insert_perform_into_def(src, "thing", "sax").is_none());
    }

    #[test]
    fn handles_nested_braces() {
        let src = "thing = { Overlay [Fm 1] | wgsl { Sm 12 } }\n";
        let out = insert_perform_into_def(src, "thing", "flute").unwrap();
        // Inserted before the OUTER closing brace, after the wgsl block.
        assert!(out.contains("wgsl { Sm 12 } | Perform(\"flute\")"));
        assert!(out.trim_end().ends_with('}'));
    }

    #[test]
    fn does_not_match_substring_name() {
        // `thing2` must not be matched when looking for `thing`.
        let src = "thing2 = { Fm 1 }\nthing = { Fm 2 }\n";
        let out = insert_perform_into_def(src, "thing", "x").unwrap();
        assert!(out.contains("thing2 = { Fm 1 }")); // untouched
        assert!(out.contains("Fm 2 | Perform(\"x\")"));
    }

    #[test]
    fn returns_none_when_missing() {
        let src = "main = { Fm 1 }\n";
        assert!(insert_perform_into_def(src, "ghost", "x").is_none());
    }
}

#[cfg(test)]
mod voice_tests {
    use super::voice_names;

    #[test]
    fn lists_audio_voices_only() {
        let src = "\
{ f: 311, l: 1, g: 1, p: 0 }
warp impasto = { Prev | Decay }
draw water = { Point }
thing1 = { Overlay [Fm 1] }
thing2 = { Fm 1/2 }
helper(x) = { Fm x }
main = { Overlay [thing1, thing2] }
";
        let v = voice_names(src);
        assert!(v.contains(&"thing1".to_string()));
        assert!(v.contains(&"thing2".to_string()));
        assert!(!v.contains(&"main".to_string()));
        assert!(!v.contains(&"impasto".to_string()));
        assert!(!v.contains(&"water".to_string()));
        assert!(!v.iter().any(|n| n.contains('('))); // no function defs
    }

    #[test]
    fn finds_the_follow_voice() {
        use super::follow_target;
        let src = "\
{ f: 311, l: 1, g: 1, p: 0 }
thing1 = { Fm 1 }
thing2 = { Fm 1/2 }
thing3 = { Fm 1 | Follow { _ -> {F, G}, } | FitLength thing1 }
main = { Overlay [thing1, thing2, thing3] }
";
        assert_eq!(follow_target(src), Some("thing3".to_string()));
    }

    #[test]
    fn no_follow_voice() {
        use super::follow_target;
        let src = "thing1 = { Fm 1 }\nmain = { thing1 }\n";
        assert_eq!(follow_target(src), None);
    }
}
