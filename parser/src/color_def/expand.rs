//! `color NAME = …` → gone, and every `| NAME` → `| Color [ … ]`.
//!
//! A source→source pass, in the family of `dsl_params` / `dsl_let` /
//! `dsl_compose`: it runs before any grammar sees the file and leaves behind
//! only text the language already understood.
//!
//! WHY EXPANSION AND NOT A SHARED ID. A colour id IS a brush's identity
//! downstream — warp chains, draw routing and the instance pool are keyed by
//! it — so two voices pointing at one id would be one brush with two voices
//! feeding it, which is how the bass got lost twice (see
//! `ColorMap::insert_unique`). `| zorn` means "the same COLOURS", not "the
//! same brush", so each site expands to its own literal list and mints its own
//! identity, exactly as if the composer had typed the hexes there. What the
//! name buys is one place to edit them and a way to derive one palette from
//! another.

use crate::color_def::ast::{ColorDef, ColorExpr, PaletteBase, PaletteExpr, PaletteOp};
use crate::color_def::color_grammar::PaletteParser;
use crate::dsl_extract::{find_matching_brace, is_ident_byte, matches_keyword, skip_ws};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ColorDefError {
    Parse { name: String, msg: String },
    UnknownPalette { in_def: String, name: String },
    BadIndex { name: String, index: usize, len: usize },
    Empty { name: String },
}

impl std::fmt::Display for ColorDefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorDefError::Parse { name, msg } => {
                write!(f, "in `color {name} = …`: {msg}")
            }
            ColorDefError::UnknownPalette { in_def, name } => write!(
                f,
                "`color {in_def}` refers to a palette named `{name}`, which is not defined \
                 (a palette must be declared before the def that uses it)"
            ),
            ColorDefError::BadIndex { name, index, len } => write!(
                f,
                "`{name}.{index}` — that palette has {len} colour(s), so its last index is {}",
                len.saturating_sub(1)
            ),
            ColorDefError::Empty { name } => {
                write!(f, "`color {name}` came out empty — a palette needs at least one colour")
            }
        }
    }
}

/// A resolved palette: plain `#rrggbb`, ready to be written back into the
/// source as a `Color [ … ]` clause.
type Resolved = Vec<String>;

fn to_rgb(spelling: &str) -> [f32; 3] {
    let c = weresocool_ast::color::parse_css_color(spelling);
    [c.r, c.g, c.b]
}

fn eval_color(
    e: &ColorExpr,
    table: &HashMap<String, Resolved>,
    in_def: &str,
) -> Result<[f32; 3], ColorDefError> {
    use weresocool_ast::color::oklab;
    Ok(match e {
        ColorExpr::Literal(s) => to_rgb(s),
        ColorExpr::Element(name, i) => {
            let p = table
                .get(name)
                .ok_or_else(|| ColorDefError::UnknownPalette {
                    in_def: in_def.to_string(),
                    name: name.clone(),
                })?;
            let hex = p.get(*i).ok_or_else(|| ColorDefError::BadIndex {
                name: name.clone(),
                index: *i,
                len: p.len(),
            })?;
            to_rgb(hex)
        }
        ColorExpr::Mix(a, b, t) => oklab::mix(
            eval_color(a, table, in_def)?,
            eval_color(b, table, in_def)?,
            *t,
        ),
        ColorExpr::Complement(c) => oklab::complement(eval_color(c, table, in_def)?),
        ColorExpr::Shade(c, t) => oklab::shade(eval_color(c, table, in_def)?, *t),
        ColorExpr::Tint(c, t) => oklab::tint(eval_color(c, table, in_def)?, *t),
        ColorExpr::Desaturate(c, t) => oklab::desaturate(eval_color(c, table, in_def)?, *t),
        ColorExpr::Rotate(c, t) => oklab::rotate_hue(eval_color(c, table, in_def)?, *t),
    })
}

fn eval_palette(
    def: &ColorDef,
    table: &HashMap<String, Resolved>,
) -> Result<Resolved, ColorDefError> {
    use weresocool_ast::color::oklab;

    let mut out: Vec<[f32; 3]> = match &def.expr.base {
        PaletteBase::List(cs) => cs
            .iter()
            .map(|c| eval_color(c, table, &def.name))
            .collect::<Result<_, _>>()?,
        PaletteBase::Named(n) => table
            .get(n)
            .ok_or_else(|| ColorDefError::UnknownPalette {
                in_def: def.name.clone(),
                name: n.clone(),
            })?
            .iter()
            .map(|h| to_rgb(h))
            .collect(),
    };

    for op in &def.expr.ops {
        match op {
            PaletteOp::Reverse => out.reverse(),
            PaletteOp::Complement => {
                out = out.into_iter().map(oklab::complement).collect()
            }
            PaletteOp::Desaturate(t) => {
                out = out.into_iter().map(|c| oklab::desaturate(c, *t)).collect()
            }
            PaletteOp::Rotate(t) => {
                out = out.into_iter().map(|c| oklab::rotate_hue(c, *t)).collect()
            }
        }
    }

    if out.is_empty() {
        return Err(ColorDefError::Empty { name: def.name.clone() });
    }
    Ok(out.iter().map(|c| oklab::to_hex(*c)).collect())
}

/// The `Color [ … ]` clause a name expands to.
fn as_clause(p: &Resolved) -> String {
    format!("Color [{}]", p.join(", "))
}

/// Pull every `color NAME = …` def out of `source`, resolve it, and rewrite
/// each `| NAME` reference into the literal clause.
pub fn expand(source: &str) -> Result<String, ColorDefError> {
    expand_with_table(source).map(|(s, _)| s)
}

/// As [`expand`], and also hand back what each name resolved to, in source
/// order, so a host can report it. A composer who writes
/// `zorn | Complement | Desaturate(0.3)` should be able to see the hexes that
/// came out without rendering.
pub fn expand_with_table(
    source: &str,
) -> Result<(String, Vec<(String, Vec<String>)>), ColorDefError> {
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut table: HashMap<String, Resolved> = HashMap::new();
    // Insertion order, for a stable report.
    let mut order: Vec<String> = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // `color` at a word boundary, followed by a NAME and an `=`. The
        // trailing name is what keeps this off the `color = vec4<f32>(…)`
        // lines inside every `Raw { … }` block in the repo: there, `color` is
        // followed straight by `=`, so there is no name and no match.
        if !matches_keyword(bytes, i, b"color") {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let block_start = i;
        let mut j = skip_ws(bytes, i + 5);
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
        let k = skip_ws(bytes, j);
        if k >= bytes.len() || bytes[k] != b'=' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        // The body runs to the end of the line, unless the list is spread over
        // several — a bracket, like a brace elsewhere, holds it open.
        let body_start = skip_ws(bytes, k + 1);
        let body_end = match bytes.get(body_start) {
            Some(b'[') => match find_matching_bracket(bytes, body_start) {
                Some(e) => e + 1,
                None => {
                    out.push(bytes[i]);
                    i += 1;
                    continue;
                }
            },
            _ => {
                let mut e = body_start;
                while e < bytes.len() && bytes[e] != b'\n' {
                    e += 1;
                }
                e
            }
        };

        let body = &source[body_start..body_end];
        let expr: PaletteExpr = PaletteParser::new()
            .parse(body)
            .map_err(|e| ColorDefError::Parse {
                name: name.clone(),
                msg: format!("{e}"),
            })?;
        let resolved = eval_palette(&ColorDef { name: name.clone(), expr }, &table)?;
        if !table.contains_key(&name) {
            order.push(name.clone());
        }
        table.insert(name, resolved);

        // Blank the def byte-for-byte so every downstream byte-span — the warp
        // promote pass above all — stays aligned with the file on disk.
        for p in block_start..body_end {
            out.push(if bytes[p] == b'\n' { b'\n' } else { b' ' });
        }
        i = body_end;
    }

    let stripped = String::from_utf8(out).expect("colour stripper preserves UTF-8");
    let report: Vec<(String, Vec<String>)> = order
        .iter()
        .map(|n| (n.clone(), table[n].clone()))
        .collect();
    if table.is_empty() {
        return Ok((stripped, report));
    }
    Ok((rewrite_refs(&stripped, &table), report))
}

/// `| zorn` → `| Color [#…, #…]`.
///
/// A bare name in a chain is how every visual def is reached — `| wet` is a
/// warp, `| form` is a draw — and a palette is no different. Only names that
/// were actually declared are touched, so a warp or draw called something else
/// passes through untouched, and line comments are copied verbatim.
fn rewrite_refs(source: &str, table: &HashMap<String, Resolved>) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        // Copy comments through without looking for refs inside them.
        if bytes[i] == b'-' && bytes.get(i + 1) == Some(&b'-')
            || bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/')
        {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }
        if bytes[i] != b'|' {
            // Multi-byte UTF-8 is copied verbatim; we only ever branch on ASCII.
            out.push_str(&source[i..i + 1]);
            i += 1;
            continue;
        }
        let after = skip_ws(bytes, i + 1);
        let name_start = after;
        let mut j = after;
        while j < bytes.len() && is_ident_byte(bytes[j]) {
            j += 1;
        }
        if j > name_start {
            if let Some(p) = table.get(&source[name_start..j]) {
                out.push_str("| ");
                out.push_str(&as_clause(p));
                i = j;
                continue;
            }
        }
        out.push('|');
        i += 1;
    }
    out
}

/// `[` … `]`, bracket-balanced.
fn find_matching_bracket(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
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

// `find_matching_brace` is imported for symmetry with the other extractors and
// used by the brace form if one is ever added; silence the unused warning.
#[allow(dead_code)]
fn _unused(b: &[u8]) -> Option<usize> {
    find_matching_brace(b, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_named_palette_expands_at_every_use() {
        let src = "color zorn = [#1a1a1a, #b8863b]\nbass = { Fm 1 | zorn }\nlead = { Fm 2 | zorn }";
        let out = expand(src).unwrap();
        assert_eq!(out.matches("Color [#1a1a1a, #b8863b]").count(), 2);
        assert!(!out.contains("color zorn"));
        // byte-for-byte blanking keeps every downstream span aligned
        assert_eq!(out.lines().count(), src.lines().count());
    }

    #[test]
    fn reverse_turns_the_ramp_end_for_end() {
        let out = expand("color a = [#000000, #ffffff]\ncolor b = a | Reverse\nx = { Fm 1 | b }")
            .unwrap();
        assert!(out.contains("Color [#ffffff, #000000]"), "{out}");
    }

    #[test]
    fn a_complement_is_a_hue_turn_not_a_negative() {
        let out = expand("color g = [#d9a441]\ncolor c = g | Complement\nx = { Fm 1 | c }").unwrap();
        let hex = out.split("Color [").nth(1).unwrap().split(']').next().unwrap();
        let rgb = to_rgb(hex);
        assert!(rgb[2] > rgb[0], "the opposite of gold is blue, got {hex}");
        // a negative would be much darker; a complement holds its weight
        let l = weresocool_ast::color::oklab::srgb_to_oklab(rgb)[0];
        let lg = weresocool_ast::color::oklab::srgb_to_oklab(to_rgb("#d9a441"))[0];
        assert!((l - lg).abs() < 0.05, "lightness moved: {l} vs {lg}");
    }

    #[test]
    fn colours_derive_from_other_palettes() {
        let out = expand(
            "color zorn = [#1a1a1a, #b8863b, #9c3a2e]\n\
             color duo = [zorn.1, mix(zorn.1, zorn.2, 0.5), complement(zorn.1)]\n\
             x = { Fm 1 | duo }",
        )
        .unwrap();
        assert!(out.contains("Color [#b8863b, "), "{out}");
        assert_eq!(out.split("Color [").nth(1).unwrap().split(',').count(), 3);
    }

    #[test]
    fn css_and_xkcd_names_still_work() {
        // Same lookup `Color [royalblue]` has always used — xkcd first, then
        // CSS — so a name means here exactly what it meant before.
        let out = expand("color c = [royalblue, black]\nx = { Fm 1 | c }").unwrap();
        assert!(out.contains("#0504aa"), "xkcd royal blue: {out}");
        assert!(out.contains("#000000"), "{out}");
    }

    #[test]
    fn a_raw_wgsl_color_assignment_is_not_a_def() {
        let src = "warp w = { Raw { color = vec4<f32>(1.0); } }";
        assert_eq!(expand(src).unwrap(), src);
    }

    #[test]
    fn an_unknown_name_is_left_for_the_dsl_that_owns_it() {
        let src = "color c = [black]\nx = { Fm 1 | wet | c | form }";
        let out = expand(src).unwrap();
        assert!(out.contains("| wet"), "{out}");
        assert!(out.contains("| form"), "{out}");
    }

    #[test]
    fn a_missing_palette_says_so() {
        let e = expand("color b = a | Reverse").unwrap_err();
        assert!(format!("{e}").contains("not defined"), "{e}");
    }

    #[test]
    fn an_out_of_range_element_names_the_length() {
        let e = expand("color a = [#000000]\ncolor b = [a.4]").unwrap_err();
        assert!(format!("{e}").contains("last index is 0"), "{e}");
    }
}
