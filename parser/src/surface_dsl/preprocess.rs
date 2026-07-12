//! Extract `surface NAME = { ... }` blocks from .socool source and
//! `| surface NAME` clauses from voice defs. Mirrors
//! `warp::preprocess::extract_warps` in shape — same scan-and-strip
//! pass over the source — but simpler because surfaces don't compose
//! into cascades (each voice routes to ONE surface, the first clause
//! it encounters in its chain).

use super::ast::{SurfaceDef, SurfacePipeline};
use super::parser::{parse_surface_pipeline, ParseError};
use crate::dsl_extract::{
    collect_refs, find_matching_brace, is_ident_byte, is_ident_start, matches_keyword, skip_ws,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct SurfacePreprocessed {
    /// Source with all `surface NAME = { ... }` defs and `| surface NAME`
    /// clauses removed. Hand off to the next preprocessor stage as-is.
    pub stripped: String,
    /// Parsed surface definitions, in source order.
    pub surfaces: Vec<SurfaceDef>,
    /// brush-index → ordered list of surface names this voice is routed to.
    /// `| surface A | surface B` accumulates both names; the voice's
    /// brushes appear on EITHER surface when that surface is active.
    /// Propagated through composition defs the same way warp routing is —
    /// if `main` routes to a surface and `main` references `rose`, then
    /// `rose`'s brush index inherits the surface alongside any of its own.
    pub color_to_surface: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub enum SurfacePreprocessError {
    UnbalancedBraces { start: usize },
    Parse { name: String, err: ParseError },
}

impl std::fmt::Display for SurfacePreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfacePreprocessError::UnbalancedBraces { start } =>
                write!(f, "unbalanced braces starting at byte {}", start),
            SurfacePreprocessError::Parse { name, err } =>
                write!(f, "surface `{}`: {}", name, err),
        }
    }
}

impl SurfacePreprocessError {
    /// If the variant carries a parse error, render it with full source
    /// context. For structural errors prints a one-liner. `quiet`
    /// suppresses output. See `warp::PreprocessError::display`.
    pub fn display(&self, quiet: bool) {
        if quiet { return; }
        match self {
            SurfacePreprocessError::Parse { name, err } => {
                eprintln!("[surface] inside `surface {} = {{ … }}`:", name);
                err.display(false);
            }
            SurfacePreprocessError::UnbalancedBraces { start } =>
                eprintln!("[surface] unbalanced braces starting at byte {}", start),
        }
    }
}

pub fn extract_surfaces(source: &str) -> Result<SurfacePreprocessed, SurfacePreprocessError> {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut surfaces: Vec<SurfaceDef> = Vec::new();
    let mut i = 0usize;

    // Pass 1: scan for `surface NAME = { ... }` blocks. Same shape as the
    // warp preprocessor — look for the `surface` keyword at a word
    // boundary, then ident, `=`, `{ … }`. Parse the body as a
    // SurfacePipeline; on success strip it from the source.
    while i < bytes.len() {
        // Skip line comments so a `surface` mentioned in a `--`/`//` comment
        // (e.g. "routed to the surface") isn't mistaken for a def header.
        if (bytes[i] == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-')
            || (bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/')
        {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }
        if matches_keyword(bytes, i, b"surface") {
            let block_start = i;
            let mut j = i + 7; // len("surface")
            j = skip_ws(bytes, j);

            let name_start = j;
            while j < bytes.len() && is_ident_byte(bytes[j]) { j += 1; }
            if j == name_start {
                // No identifier — fall through as plain text.
                out.push(bytes[i] as char); i += 1; continue;
            }
            let name = source[name_start..j].to_string();

            // `=`
            let k = skip_ws(bytes, j);
            if k >= bytes.len() || bytes[k] != b'=' {
                out.push(bytes[i] as char); i += 1; continue;
            }
            let k = skip_ws(bytes, k + 1);

            // `{`
            if k >= bytes.len() || bytes[k] != b'{' {
                out.push(bytes[i] as char); i += 1; continue;
            }
            let body_start = k + 1;
            let body_end = match find_matching_brace(bytes, k) {
                Some(e) => e,
                None => return Err(SurfacePreprocessError::UnbalancedBraces { start: block_start }),
            };

            let body = &source[body_start..body_end];
            let pipeline = parse_surface_pipeline(body)
                .map_err(|err| SurfacePreprocessError::Parse { name: name.clone(), err })?;
            surfaces.push(SurfaceDef { name, pipeline });

            // Preserve newlines so downstream parse errors still report
            // the right line. Without this, a multi-line `surface NAME = { ... }`
            // strips N newlines and the weresocool parser reports errors
            // N lines earlier than they really are. Mirrors warp's stripper.
            let block_end = body_end + 1; // include the `}`
            for p in i..block_end {
                out.push(if bytes[p] == b'\n' { '\n' } else { ' ' });
            }
            i = block_end;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }

    // Pass 2: strip `| surface NAME` clauses from voice bodies and collect
    // brush-idx → surface routing. Same shape as the warp def walker.
    let surface_names: HashSet<String> =
        surfaces.iter().map(|s| s.name.clone()).collect();
    let (final_stripped, color_to_surface, inline_surfaces) =
        strip_surface_chain_ops(&out, &surface_names);
    // Inline `| surface { … }` blocks become first-class synthetic defs.
    surfaces.extend(inline_surfaces);

    Ok(SurfacePreprocessed {
        stripped: final_stripped,
        surfaces,
        color_to_surface,
    })
}

struct DefInfo {
    body_start: usize,
    body_end: usize,
    has_color: bool,
    /// Every surface clause this def's body carries, in source order
    /// (named refs + inline blocks). Multiple clauses accumulate — a voice
    /// with `| circle | surface plane` shows on EITHER surface.
    surface_tags: Vec<SurfaceTag>,
    refs: Vec<String>,
    stripped_body: String,
}

fn strip_surface_chain_ops(
    source: &str,
    surface_names: &HashSet<String>,
) -> (String, HashMap<String, Vec<String>>, Vec<SurfaceDef>) {
    let bytes = source.as_bytes();

    // Pass 1: collect every top-level `name = { body }` and scan the body
    // for all `| surface NAME` clauses.
    let mut def_order: Vec<String> = Vec::new();
    let mut defs: HashMap<String, DefInfo> = HashMap::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let line_start = i == 0 || bytes[i - 1] == b'\n';
        if line_start && is_ident_start(bytes[i]) {
            let name_start = i;
            while i < bytes.len() && is_ident_byte(bytes[i]) { i += 1; }
            let name = source[name_start..i].to_string();
            let j = skip_ws(bytes, i);
            if j < bytes.len() && bytes[j] == b'=' {
                let k = skip_ws(bytes, j + 1);
                if k < bytes.len() && bytes[k] == b'{' {
                    if let Some(body_end) = find_matching_brace(bytes, k) {
                        let body = &source[k + 1..body_end];
                        let (stripped_body, surface_tags, has_color) =
                            scan_def_body(body, surface_names);
                        if !defs.contains_key(&name) {
                            def_order.push(name.clone());
                        }
                        defs.insert(name, DefInfo {
                            body_start: k + 1,
                            body_end,
                            has_color,
                            surface_tags,
                            refs: Vec::new(),
                            stripped_body,
                        });
                        i = body_end + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    // Pass 2: build the def→def reference graph.
    let def_names: HashSet<String> = defs.keys().cloned().collect();
    for info in defs.values_mut() {
        let body = &source[info.body_start..info.body_end];
        info.refs = collect_refs(body, &def_names);
    }

    // Pass 3: assign brush indices in source order. Every def with a Color
    // clause gets the next sequential index — same convention as warps.
    let mut name_to_brush_idx: HashMap<String, u32> = HashMap::new();
    let mut next_idx = 0u32;
    for name in &def_order {
        if defs.get(name).map_or(false, |d| d.has_color) {
            name_to_brush_idx.insert(name.clone(), next_idx);
            next_idx += 1;
        }
    }

    // Pass 3b: materialize inline `| surface { … }` blocks into synthetic
    // named defs, and resolve every def's tags down to a flat name list.
    let mut inline_surfaces: Vec<SurfaceDef> = Vec::new();
    let mut names_for_def: HashMap<String, Vec<String>> = HashMap::new();
    for name in &def_order {
        let Some(info) = defs.get(name) else { continue };
        let mut resolved = Vec::with_capacity(info.surface_tags.len());
        for (i, tag) in info.surface_tags.iter().enumerate() {
            resolved.push(match tag {
                SurfaceTag::Named(n) => n.clone(),
                SurfaceTag::Inline(pipe) => {
                    let anon = format!("__inline_surface_{}_{}", name, i);
                    inline_surfaces.push(SurfaceDef { name: anon.clone(), pipeline: pipe.clone() });
                    anon
                }
            });
        }
        names_for_def.insert(name.clone(), resolved);
    }

    // Pass 4: for each leaf brush def, accumulate its full surface list —
    // its own clauses PLUS any inherited from composition defs that
    // transitively reach it. Consecutive duplicates collapse.
    let mut index_to_surfaces: HashMap<String, Vec<String>> = HashMap::new();
    for leaf_name in &def_order {
        let Some(leaf_info) = defs.get(leaf_name) else { continue };
        if !leaf_info.has_color { continue }
        let Some(&idx) = name_to_brush_idx.get(leaf_name) else { continue };

        let mut accum: Vec<String> = names_for_def.get(leaf_name).cloned().unwrap_or_default();
        for comp_name in &def_order {
            if comp_name == leaf_name { continue }
            let Some(comp_info) = defs.get(comp_name) else { continue };
            if comp_info.has_color { continue }
            let comp_names = names_for_def.get(comp_name).map(|v| v.as_slice()).unwrap_or(&[]);
            if comp_names.is_empty() { continue }
            if reaches(comp_name, leaf_name, &defs) {
                accum.extend(comp_names.iter().cloned());
            }
        }
        // Dedup consecutive duplicates only — `| surface A | surface A`
        // collapses but `| surface A | surface B | surface A` stays.
        accum.dedup();
        if !accum.is_empty() {
            index_to_surfaces.insert(idx.to_string(), accum);
        }
    }

    // Pass 5: reassemble the source with each def's body replaced by its
    // stripped version.
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut by_pos: Vec<(&String, &DefInfo)> = defs.iter().collect();
    by_pos.sort_by_key(|(_, d)| d.body_start);
    for (_name, info) in &by_pos {
        out.push_str(&source[cursor..info.body_start]);
        out.push_str(&info.stripped_body);
        out.push('}');
        cursor = info.body_end + 1;
    }
    out.push_str(&source[cursor..]);

    (out, index_to_surfaces, inline_surfaces)
}

/// A surface clause in a voice body: a named ref (bare `| name` or keyword
/// `| surface name`) or an inline `| surface { … }` block (parsed pipeline,
/// materialized into a synthetic def later).
enum SurfaceTag {
    Named(String),
    Inline(SurfacePipeline),
}

fn scan_def_body(body: &str, surface_names: &HashSet<String>) -> (String, Vec<SurfaceTag>, bool) {
    // All three forms now: bare `| name`, keyword `| surface name`, and inline
    // `| surface { … }`. Multiple clauses chain (a voice can sit on several
    // surfaces).
    let scan = crate::dsl_extract::scan_def_body(body, b"surface", surface_names, /* allow_inline = */ true);
    let mut tags = Vec::with_capacity(scan.tags.len());
    for t in scan.tags {
        match t {
            crate::dsl_extract::RefTag::Named(n) => tags.push(SurfaceTag::Named(n)),
            crate::dsl_extract::RefTag::Inline { body, .. } => {
                // Drop a malformed inline (clause still elided) — no shipped
                // composition has one, matching draw's behavior.
                if let Ok(p) = parse_surface_pipeline(&body) {
                    tags.push(SurfaceTag::Inline(p));
                }
            }
        }
    }
    (scan.stripped, tags, scan.has_color)
}

fn reaches(from: &str, to: &str, defs: &HashMap<String, DefInfo>) -> bool {
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = vec![from.to_string()];
    while let Some(d) = stack.pop() {
        if d == to { return true; }
        if !visited.insert(d.clone()) { continue }
        if let Some(info) = defs.get(&d) {
            for r in &info.refs { stack.push(r.clone()); }
        }
    }
    false
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_surface_def() {
        // Top-level defs must start at column 0 (matches the .socool
        // convention used by the warp preprocessor).
        let src = "surface ripple = {\n\
            Plane(10, 10, 128)\n\
            | Wave2(0.7, 0.5, 0.45)\n\
            | Smooth\n\
        }\nx = 1\n";
        let p = extract_surfaces(src).unwrap();
        assert_eq!(p.surfaces.len(), 1);
        assert_eq!(p.surfaces[0].name, "ripple");
        assert!(!p.stripped.contains("surface ripple"));
    }

    #[test]
    fn extracts_surface_routing() {
        let src = "surface ripple = { Plane(10, 10, 64) }\n\
            rose = {\n\
                Sine\n\
                | Color [red]\n\
                | surface ripple\n\
            }\n";
        let p = extract_surfaces(src).unwrap();
        assert_eq!(p.color_to_surface.get("0"), Some(&vec!["ripple".to_string()]));
        assert!(!p.stripped.contains("| surface ripple"));
    }

    #[test]
    fn surface_word_in_comment_is_not_a_def() {
        // `surface` in a `--` comment right before a def must not be parsed as
        // a `surface NAME = {…}` header.
        let src = "surface ripple = { Plane(8, 8, 64) }\n\
            -- this voice is routed to the surface\n\
            rose = {\n\
                Sine\n\
                | Color [red]\n\
                | ripple\n\
            }\n";
        let p = extract_surfaces(src).expect("must not mis-parse the comment");
        assert_eq!(p.surfaces.len(), 1);
        assert_eq!(p.surfaces[0].name, "ripple");
        assert_eq!(p.color_to_surface.get("0"), Some(&vec!["ripple".to_string()]));
    }

    #[test]
    fn inline_surface_block_materializes() {
        // `| surface { … }` becomes a synthetic def the brush routes to —
        // full bare/keyword/inline parity with draw/warp.
        let src = "rose = {\n\
                Sine\n\
                | Color [red]\n\
                | surface { Plane(10, 10, 64) }\n\
            }\n";
        let p = extract_surfaces(src).unwrap();
        assert_eq!(p.surfaces.len(), 1, "one synthesized surface def");
        let anon = &p.surfaces[0].name;
        assert!(anon.starts_with("__inline_surface_"), "got {anon}");
        assert_eq!(p.color_to_surface.get("0"), Some(&vec![anon.clone()]));
        assert!(!p.stripped.contains("surface {"));
    }

    #[test]
    fn bare_surface_ref_without_keyword() {
        // Bare `| ripple` (no `surface` keyword) now routes the same as
        // `| surface ripple`, consistent with warp/draw. Audio ops are
        // untouched (the `surface_names` gate).
        let src = "surface ripple = { Plane(10, 10, 64) }\n\
            rose = {\n\
                Sine\n\
                | Color [red]\n\
                | ripple\n\
            }\n";
        let p = extract_surfaces(src).unwrap();
        assert_eq!(p.color_to_surface.get("0"), Some(&vec!["ripple".to_string()]));
        assert!(!p.stripped.contains("| ripple"));
    }

    #[test]
    fn extracts_recursive_routing() {
        // `| surface A | surface B` puts the voice on BOTH surfaces.
        let src = "surface circle = { Plane(8, 8, 64) }\n\
            surface plane = { Plane(10, 10, 64) }\n\
            rose = {\n\
                Sine\n\
                | Color [red]\n\
                | surface circle\n\
                | surface plane\n\
            }\n";
        let p = extract_surfaces(src).unwrap();
        assert_eq!(
            p.color_to_surface.get("0"),
            Some(&vec!["circle".to_string(), "plane".to_string()])
        );
    }
}
