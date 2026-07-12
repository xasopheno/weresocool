//! `palette { … }` extraction — kintaro DSL extension (like `warp`/`draw`/
//! `surface`), stripped before weresocool parses.
//!
//! A palette is a set of named looper *instruments* (sound + color + draw) the
//! DAW can record into. They are NOT part of the played composition — they're
//! templates. Syntax (entries are plain weresocool defs, no commas):
//!
//! ```text
//! palette {
//!   fifths = { Overlay [Fm 1, Fm 3/2] | Color [red] | water }
//!   bass   = { Fm 1/2 | Color [midnightblue] }
//! }
//! ```
//!
//! Extraction strips the `palette { … }` wrapper and leaves the inner defs as
//! ordinary top-level defs (so they're usable as `Id`s by injected layers), and
//! reports the palette names for the DAW. The defs are never added to `main`, so
//! they stay silent until a layer references one.
//!
//! IMPORTANT: a palette sound is a DECORATOR — `Perform` multiplies EVERY op of
//! it against every recorded note. Keep sounds to a chord/single-note shape
//! (an `Overlay` of members is perfect); a `| Repeat N` or long `Seq` inside
//! one multiplies the layer's voices N-fold (N× louder, N× render cost).

#[derive(Debug, Clone, Default)]
pub struct PaletteExtract {
    /// Source with the `palette { … }` wrapper removed (inner defs hoisted to
    /// top level).
    pub stripped: String,
    /// Palette instrument names, in source order.
    pub names: Vec<String>,
    /// Optional palette name (`palette NAME { … }`) — the DAW uses it as the
    /// slot the user drops into `main` where recorded layers sit. `None` for the
    /// unnamed `palette { … }` form (slot name defaults to `daw`).
    pub name: Option<String>,
}

/// Strip the first `palette { … }` block (if any) and collect its def names.
pub fn extract_palette(source: &str) -> PaletteExtract {
    let bytes = source.as_bytes();
    let Some(kw) = find_keyword(source, "palette") else {
        return PaletteExtract::default_passthrough(source);
    };
    // Optional name, then the first `{`, between the keyword and the brace.
    let mut i = kw + "palette".len();
    // skip whitespace
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    // read an identifier if present (the palette name)
    let name = {
        let start = i;
        while i < bytes.len() && is_ident_byte(bytes[i]) {
            i += 1;
        }
        (i > start).then(|| source[start..i].to_string())
    };
    while i < bytes.len() && bytes[i] != b'{' {
        i += 1;
    }
    let Some(open) = (i < bytes.len()).then_some(i) else {
        return PaletteExtract::default_passthrough(source);
    };
    let Some(close) = matching_brace(bytes, open) else {
        return PaletteExtract::default_passthrough(source);
    };

    let inner = &source[open + 1..close];
    let names = def_names(inner);

    // Replace `palette [NAME] { … }` with the inner defs (hoisted to top level).
    let mut stripped = String::with_capacity(source.len());
    stripped.push_str(&source[..kw]);
    stripped.push_str(inner);
    stripped.push_str(&source[close + 1..]);

    PaletteExtract { stripped, names, name }
}

impl PaletteExtract {
    fn default_passthrough(source: &str) -> PaletteExtract {
        PaletteExtract { stripped: source.to_string(), names: Vec::new(), name: None }
    }
}

/// Find `word` as a standalone token at the START of a line (only whitespace
/// before it on that line) — so a `palette {` block is matched but the word
/// "palette" inside a comment or expression is not.
fn find_keyword(source: &str, word: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut from = 0;
    while let Some(rel) = source[from..].find(word) {
        let at = from + rel;
        let end = at + word.len();
        let next_ok = bytes.get(end).map_or(true, |&c| !is_ident_byte(c));
        // Line-start: everything from the previous newline to `at` is whitespace.
        let mut k = at;
        let mut line_start = true;
        while k > 0 && bytes[k - 1] != b'\n' {
            if !bytes[k - 1].is_ascii_whitespace() {
                line_start = false;
                break;
            }
            k -= 1;
        }
        if line_start && next_ok {
            return Some(at);
        }
        from = end;
    }
    None
}

/// Top-level `name = { … }` def names within `inner` (depth-0 only).
fn def_names(inner: &str) -> Vec<String> {
    let bytes = inner.as_bytes();
    let mut names = Vec::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            c if depth == 0 && (c == b'_' || c.is_ascii_lowercase()) => {
                // Possible identifier start; read it.
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let name = &inner[start..i];
                // Expect `=` then `{` (whitespace allowed).
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if bytes.get(j) == Some(&b'=') {
                    j += 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'{') {
                        names.push(name.to_string());
                    }
                }
                continue; // already advanced i past the identifier
            }
            _ => {}
        }
        i += 1;
    }
    names
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

fn is_ident_byte(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric() || c == b'.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_names_and_hoists_defs() {
        let src = "\
{ f: 311, l: 1, g: 1, p: 0 }
palette {
  fifths = { Overlay [Fm 1, Fm 3/2] }
  bass   = { Fm 1/2 }
}
thing1 = { Fm 1 }
main = { thing1 }
";
        let p = extract_palette(src);
        assert_eq!(p.names, vec!["fifths".to_string(), "bass".to_string()]);
        // Wrapper gone, inner defs hoisted, rest intact.
        assert!(!p.stripped.contains("palette {"));
        assert!(p.stripped.contains("fifths = { Overlay [Fm 1, Fm 3/2] }"));
        assert!(p.stripped.contains("bass   = { Fm 1/2 }"));
        assert!(p.stripped.contains("thing1 = { Fm 1 }"));
        assert!(p.stripped.contains("main = { thing1 }"));
    }

    #[test]
    fn parses_optional_name() {
        let unnamed = extract_palette("palette {\n fifths = { Fm 1 }\n}\nmain = { thing1 }\n");
        assert_eq!(unnamed.name, None);
        assert_eq!(unnamed.names, vec!["fifths".to_string()]);

        let named = extract_palette("palette perf {\n fifths = { Fm 1 }\n}\nmain = { thing1 }\n");
        assert_eq!(named.name.as_deref(), Some("perf"));
        assert_eq!(named.names, vec!["fifths".to_string()]);
        assert!(!named.stripped.contains("palette"));
        assert!(named.stripped.contains("fifths = { Fm 1 }"));
    }

    #[test]
    fn no_palette_is_passthrough() {
        let src = "thing1 = { Fm 1 }\nmain = { thing1 }\n";
        let p = extract_palette(src);
        assert!(p.names.is_empty());
        assert_eq!(p.stripped, src);
    }
}
