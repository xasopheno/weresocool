//! Extract `draw NAME = { … }` blocks from .socool source before LALRPOP sees it.
//!
//! Mirrors `crate::warp::preprocess` exactly in shape. Differences:
//! - No blend modes (draw produces instances, not a composited frame).
//! - Per-brush routing is a single `draw` name (no cascade) — the *last* draw
//!   clause that reaches the leaf wins. Composers who want multiple drawings
//!   layered on one brush can wrap them in a single named draw using `Each`.
//!
//! See `docs/draw-dsl-design.md`.

use super::ast::{DrawDef, DrawPipeline};
use super::parser_lalrpop::ParseError;
use super::parser_lalrpop::parse_pipeline_lalrpop as parse_pipeline;
use crate::dsl_extract::{
    collect_refs, find_matching_brace, is_ident_byte, is_ident_start, matches_keyword, skip_ws,
};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
enum DrawTagSource {
    Named(String),
    Inline(DrawPipeline),
}

#[derive(Debug)]
pub struct Preprocessed {
    /// Source with `draw NAME = { … }` blocks, `| draw { … }` inline blocks,
    /// and `| <draw_name>` chain ops removed. Top-level blocks are blanked
    /// one space per source byte (newlines kept) so byte offsets stay
    /// aligned for downstream errors; chain ops and inline blocks are
    /// elided by `dsl_extract::scan_def_body` (newlines kept, so LINE
    /// numbers stay aligned even where byte offsets shift).
    pub stripped: String,
    /// Parsed draw definitions in source order (includes synthesized
    /// `__inline_<def>_<i>` blocks materialized from inline forms).
    pub draws: Vec<DrawDef>,
    /// brush-index → name of the draw to run for this brush. Late binding to
    /// the `draws` Vec by name. Missing means "no draw — use the default
    /// 1-note-1-instance dispatch path."
    pub color_to_draw: HashMap<String, String>,
}

#[derive(Debug)]
pub enum PreprocessError {
    UnbalancedBraces { start: usize },
    Parse { name: String, err: ParseError },
}

impl PreprocessError {
    /// If the variant carries a parse error, render it with full source
    /// context. For structural errors prints a one-liner. `quiet`
    /// suppresses output. See `warp::PreprocessError::display`.
    pub fn display(&self, quiet: bool) {
        if quiet { return; }
        match self {
            PreprocessError::Parse { name, err } => {
                eprintln!("[draw] inside `draw {} = {{ … }}`:", name);
                err.display(false);
            }
            PreprocessError::UnbalancedBraces { start } =>
                eprintln!("[draw] unbalanced braces starting at byte {}", start),
        }
    }
}

pub fn extract_draws(source: &str) -> Result<Preprocessed, PreprocessError> {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut draws = Vec::new();
    let mut i = 0usize;
    // Pass-through is copied as string SLICES between edit points (never
    // byte-by-byte `as char` casts, which would re-encode UTF-8 bytes ≥ 0x80
    // as two-byte mojibake and break the byte-offset invariant documented on
    // `Preprocessed::stripped`). `cursor` marks the start of the pending
    // untouched slice.
    let mut cursor = 0usize;

    while i < bytes.len() {
        if matches_keyword(bytes, i, b"draw") {
            let block_start = i;
            let mut j = i + 4;
            j = skip_ws(bytes, j);

            // Identifier.
            let name_start = j;
            while j < bytes.len() && is_ident_byte(bytes[j]) { j += 1; }
            if j == name_start {
                // Not a `draw NAME = …` block (could be `| draw { … }` inline
                // form — handled in pass 2 below). Don't eat the keyword.
                i += 1;
                continue;
            }
            let name = source[name_start..j].to_string();

            // `=`
            let k = skip_ws(bytes, j);
            if k >= bytes.len() || bytes[k] != b'=' {
                i += 1; continue;
            }
            let k = skip_ws(bytes, k + 1);

            // `{`
            if k >= bytes.len() || bytes[k] != b'{' {
                i += 1; continue;
            }
            let body_start = k + 1;
            let body_end = match find_matching_brace(bytes, k) {
                Some(e) => e,
                None => return Err(PreprocessError::UnbalancedBraces { start: block_start }),
            };

            let body = &source[body_start..body_end];
            let pipeline = parse_pipeline(body)
                .map_err(|err| PreprocessError::Parse { name: name.clone(), err })?;
            draws.push(DrawDef { name, pipeline });

            // Flush everything untouched up to the block start.
            out.push_str(&source[cursor..i]);
            // Preserve newlines so error spans downstream of the stripped
            // block still point at the right line in the original source.
            // Otherwise a multi-line `draw NAME = { ... }` swallows N
            // newlines and weresocool reports parse errors N lines earlier
            // than they really are. See the parallel logic in
            // warp::preprocess::extract_warps. Blanking is per-BYTE (one
            // space per source byte) so byte offsets stay aligned even if
            // the block body contained multi-byte UTF-8.
            let block_end = body_end + 1; // include the `}`
            for p in i..block_end {
                out.push(if bytes[p] == b'\n' { '\n' } else { ' ' });
            }
            i = block_end;
            cursor = block_end;
        } else {
            i += 1;
        }
    }
    out.push_str(&source[cursor..]);

    let name_set: HashSet<String> = draws.iter().map(|d| d.name.clone()).collect();
    let (stripped, color_to_draw, inline_draws) =
        strip_chain_draw_ops(&out, &name_set);

    let mut all = draws;
    all.extend(inline_draws);

    Ok(Preprocessed {
        stripped,
        draws: all,
        color_to_draw,
    })
}

struct DefInfo {
    body_start: usize,
    body_end: usize,
    has_color: bool,
    /// Draw clauses found in this def's body, in source order. For v1 the
    /// effective draw per brush is the *last* one in source order (no cascade).
    draw_tags: Vec<DrawTagSource>,
    refs: Vec<String>,
    stripped_body: String,
}

fn strip_chain_draw_ops(
    source: &str,
    draw_names: &HashSet<String>,
) -> (String, HashMap<String, String>, Vec<DrawDef>) {
    let bytes = source.as_bytes();

    // Pass 1: collect top-level let bindings.
    let mut def_order: Vec<String> = Vec::new();
    let mut defs: HashMap<String, DefInfo> = HashMap::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let line_start = i == 0 || bytes[i - 1] == b'\n';
        if line_start {
            // Skip leading spaces/tabs so INDENTED top-level defs are found:
            // palette-block brushes are hoisted to top level later but keep
            // their indentation, and their `| <draw>` clauses still need routing.
            let mut s = i;
            while s < bytes.len() && (bytes[s] == b' ' || bytes[s] == b'\t') {
                s += 1;
            }
            if s < bytes.len() && is_ident_start(bytes[s]) {
                let name_start = s;
                let mut e = s;
                while e < bytes.len() && is_ident_byte(bytes[e]) { e += 1; }
                let name = source[name_start..e].to_string();
                let j = skip_ws(bytes, e);
                if j < bytes.len() && bytes[j] == b'=' {
                    let k = skip_ws(bytes, j + 1);
                    if k < bytes.len() && bytes[k] == b'{' {
                        if let Some(body_end) = find_matching_brace(bytes, k) {
                            let body = &source[k + 1..body_end];
                            let (stripped_body, draw_tags, has_color) =
                                scan_def_body(body, draw_names);
                            if !defs.contains_key(&name) {
                                def_order.push(name.clone());
                            }
                            defs.insert(name, DefInfo {
                                body_start: k + 1,
                                body_end,
                                has_color,
                                draw_tags,
                                refs: Vec::new(),
                                stripped_body,
                            });
                            i = body_end + 1;
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    // Pass 2: discover def references.
    let def_names: HashSet<String> = defs.keys().cloned().collect();
    for info in defs.values_mut() {
        let body = &source[info.body_start..info.body_end];
        info.refs = collect_refs(body, &def_names);
    }

    // Pass 3: assign brush indices in source order (matches weresocool's own
    // assignment for `Color [...]` clauses).
    let mut name_to_brush_idx: HashMap<String, u32> = HashMap::new();
    let mut next_idx = 0u32;
    for name in &def_order {
        if defs.get(name).map_or(false, |d| d.has_color) {
            name_to_brush_idx.insert(name.clone(), next_idx);
            next_idx += 1;
        }
    }

    // Pass 4: materialize inline `| draw { … }` blocks into synthetic names.
    let mut inline_draws: Vec<DrawDef> = Vec::new();
    let mut names_for_def: HashMap<String, Vec<String>> = HashMap::new();
    for name in &def_order {
        let Some(info) = defs.get(name) else { continue };
        let mut resolved: Vec<String> = Vec::with_capacity(info.draw_tags.len());
        for (i, tag) in info.draw_tags.iter().enumerate() {
            resolved.push(match tag {
                DrawTagSource::Named(n) => n.clone(),
                DrawTagSource::Inline(pipe) => {
                    let anon = format!("__inline_draw_{}_{}", name, i);
                    inline_draws.push(DrawDef { name: anon.clone(), pipeline: pipe.clone() });
                    anon
                }
            });
        }
        names_for_def.insert(name.clone(), resolved);
    }

    // Pass 4b: append a `#__kdraw_<name>` tag to every def that carries a draw
    // clause (its own last one wins, mirroring Pass 5). The scanner already
    // elided the original `| <draw>` from the body; this tag rides the def's
    // ops through Perform / DAW injection so the draw resolves at render by
    // TAG (color-id-order-independent) rather than by the fragile brush index.
    for name in &def_order {
        let Some(last) = names_for_def.get(name).and_then(|r| r.last()).cloned() else {
            continue;
        };
        if let Some(info) = defs.get_mut(name) {
            info.stripped_body
                .push_str(&format!(" | #{}{}", super::TAG_PREFIX, last));
        }
    }

    // Pass 5: for each leaf brush, pick the effective draw — leaf's own last
    // clause if any, else the deepest-source-order composition def that
    // reaches it.
    let mut index_to_draw: HashMap<String, String> = HashMap::new();
    for leaf_name in &def_order {
        let Some(leaf_info) = defs.get(leaf_name) else { continue };
        if !leaf_info.has_color { continue }
        let Some(&idx) = name_to_brush_idx.get(leaf_name) else { continue };

        // Leaf's own draw wins if present.
        if let Some(names) = names_for_def.get(leaf_name) {
            if let Some(last) = names.last() {
                index_to_draw.insert(idx.to_string(), last.clone());
                continue;
            }
        }
        // Otherwise scan composition defs whose ref-graph transitively
        // contains this leaf. Last-in-source-order wins.
        for comp_name in def_order.iter().rev() {
            if comp_name == leaf_name { continue }
            let Some(comp_info) = defs.get(comp_name) else { continue };
            if comp_info.has_color { continue }
            let Some(comp_names) = names_for_def.get(comp_name) else { continue };
            let Some(last) = comp_names.last() else { continue };
            if reaches(comp_name, leaf_name, &defs) {
                index_to_draw.insert(idx.to_string(), last.clone());
                break;
            }
        }
    }

    // Pass 6: reassemble the source, replacing each def body with its stripped form.
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut by_pos: Vec<(&String, &DefInfo)> = defs.iter().collect();
    by_pos.sort_by_key(|(_, d)| d.body_start);
    for (_name, info) in &by_pos {
        out.push_str(&source[cursor..info.body_start]); // includes `{`
        out.push_str(&info.stripped_body);
        out.push('}');
        cursor = info.body_end + 1;
    }
    out.push_str(&source[cursor..]);

    (out, index_to_draw, inline_draws)
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

fn scan_def_body(body: &str, draw_names: &HashSet<String>) -> (String, Vec<DrawTagSource>, bool) {
    let scan = crate::dsl_extract::scan_def_body(body, b"draw", draw_names, /* allow_inline = */ true);
    let mut tags = Vec::with_capacity(scan.tags.len());
    for t in scan.tags {
        match t {
            crate::dsl_extract::RefTag::Named(n) => tags.push(DrawTagSource::Named(n)),
            // Inline `| draw { … }` — parse the body into a pipeline. A
            // malformed inline is still elided from the audio (so playback
            // continues), but the composer must HEAR about it — a silently
            // dropped draw clause looks like "my visuals vanished".
            crate::dsl_extract::RefTag::Inline { body, .. } => {
                match parse_pipeline(&body) {
                    Ok(pipe) => tags.push(DrawTagSource::Inline(pipe)),
                    Err(e) => eprintln!(
                        "[draw] inline `| draw {{ … }}` block failed to parse and was \
                         skipped (brush falls back to default rendering): {e:?}"
                    ),
                }
            }
        }
    }
    (scan.stripped, tags, scan.has_color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_top_level_draw_block() {
        let src = "
draw ring = { Point | Xa(0.3) | Spawn(8, by: Rz(1/8)) }
voice = { Color [red, blue] | draw ring }
";
        let pre = extract_draws(src).expect("preprocess");
        assert_eq!(pre.draws.len(), 1);
        assert_eq!(pre.draws[0].name, "ring");
        // brush 0 should route to `ring`.
        assert_eq!(pre.color_to_draw.get("0"), Some(&"ring".to_string()));
        // The stripped source must not contain `draw ring` (the block is gone).
        assert!(!pre.stripped.contains("draw ring"));
    }

    #[test]
    fn bare_draw_ref_without_keyword() {
        // `| ring` with no `draw` keyword routes the same as `| draw ring`,
        // mirroring warp's bare `| warp_name`. Audio ops in the same chain
        // (here `Gain`) must NOT be swallowed.
        let src = "
draw ring = { Point | Xa(0.3) | Spawn(8, by: Rz(1/8)) }
voice = { Color [red, blue] | Gain 1/2 | ring }
";
        let pre = extract_draws(src).expect("preprocess");
        assert_eq!(pre.color_to_draw.get("0"), Some(&"ring".to_string()));
        // The draw ref is stripped, but the audio op survives.
        assert!(!pre.stripped.contains("| ring"));
        assert!(pre.stripped.contains("Gain 1/2"));
    }

    #[test]
    fn routes_inline_draw_block() {
        let src = "
voice = { Color [red] | draw { Point | Xa(0.3) | Spawn(8, by: Rz(1/8)) } }
";
        let pre = extract_draws(src).expect("preprocess");
        // One synthesized inline def.
        assert_eq!(pre.draws.len(), 1);
        assert!(pre.draws[0].name.starts_with("__inline_draw_voice_"));
        // Brush 0 routes to it.
        assert_eq!(
            pre.color_to_draw.get("0"),
            Some(&pre.draws[0].name)
        );
    }

    #[test]
    fn non_ascii_passthrough_preserves_bytes() {
        // UTF-8 outside draw blocks must pass through untouched — the old
        // byte-by-byte `as char` pass-through re-encoded bytes ≥ 0x80 as
        // Latin-1 mojibake (each source byte became TWO output bytes),
        // corrupting content and breaking the byte-offset invariant.
        // (No `| draw name` chain op here: chain-op elision is length-
        // changing by design; this test pins the block-blanking path.)
        let src = "
-- “détail” — smart quotes and é
draw ring = { Point | Xa(0.3) | Spawn(8, by: Rz(1/8)) }
voice = { Color [red] }
-- après
";
        let pre = extract_draws(src).expect("preprocess");
        // Byte-offset invariant: output length equals input length (the
        // draw block is blanked one space per source byte).
        assert_eq!(pre.stripped.len(), src.len(), "byte offsets must stay aligned");
        // Content outside the draw block is unchanged.
        assert!(pre.stripped.contains("“détail” — smart quotes and é"));
        assert!(pre.stripped.contains("après"));
        assert!(pre.stripped.contains("Color [red]"));
        // The draw block itself is still extracted and blanked out.
        assert_eq!(pre.draws.len(), 1);
        assert_eq!(pre.draws[0].name, "ring");
        assert!(!pre.stripped.contains("draw ring"));
    }

    #[test]
    fn propagates_through_composition() {
        let src = "
draw ring = { Point | Xa(0.3) | Spawn(8, by: Rz(1/8)) }
voice = { Color [red] }
main = { voice | draw ring }
";
        let pre = extract_draws(src).expect("preprocess");
        // The leaf has no own clause, but `main` reaches it.
        assert_eq!(pre.color_to_draw.get("0"), Some(&"ring".to_string()));
    }
}
