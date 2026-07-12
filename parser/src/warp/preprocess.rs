//! Extract `warp NAME = { ... }` blocks from .socool source before LALRPOP sees it.
//!
//! Mirrors how the external weresocool crate handles `wgsl { ... }` blocks
//! (regex-find + body-extract + replace before parsing). Warp blocks are
//! orthogonal to the rest of the AST — we just elide them from the source
//! and return the parsed defs alongside.

use super::ast::{WarpBlendMode, WarpDef, WarpPipeline};
use super::parser_lalrpop::ParseError;
use super::parser_lalrpop::parse_pipeline_lalrpop as parse_pipeline;
use crate::dsl_extract::{
    collect_refs, find_matching_brace, is_ident_byte, is_ident_start, matches_keyword, skip_ws,
};
use std::collections::{HashMap, HashSet};

/// Either a named warp reference (`| plasma`) or an inline anonymous block
/// (`| warp { ... }`) discovered inside a def body.
#[derive(Clone)]
enum WarpTagSource {
    Named(String),
    /// Inline anonymous `| warp { … }`. Carries the parsed pipeline AND
    /// the body's byte range RELATIVE TO THE DEF BODY it was found in.
    /// Materialization lifts this to a global span via
    /// `def.body_start + body_span.start` so the promote-pass can scope
    /// its literal scan correctly. Without this the inline warp's
    /// tweakable params would be invisible to the UI.
    Inline {
        pipeline: WarpPipeline,
        /// `(start, end)` within the enclosing def's body slice.
        body_span: (usize, usize),
    },
}

#[derive(Debug)]
pub struct Preprocessed {
    /// Source with all `warp NAME = { ... }` blocks, `| warp { ... }` inline
    /// blocks, and `| <warp>` chain ops removed.
    pub stripped: String,
    /// Parsed warp definitions, in source order (includes inline anonymous warps).
    pub warps: Vec<WarpDef>,
    /// brush-index → chain of warp names, in cascade order. The first warp
    /// receives the brush's scene; each subsequent warp takes the previous
    /// warp's output as its scene. Built by walking the def-reference graph:
    /// each leaf's chain is its own tags followed by tags inherited from
    /// every composition def that reaches it (source-order).
    pub color_to_chain: HashMap<String, Vec<String>>,
    /// warp name → its declared blend mode (`warp NAME <mode> = {...}`).
    /// Absent = Additive. Used by the compositor when this warp is a chain's
    /// final stage.
    pub warp_blend_modes: HashMap<String, WarpBlendMode>,
    /// Audio def name → brush index (source order of Color-clause defs,
    /// matching weresocool's BrushIdentifier numbering). Lets downstream
    /// consumers resolve PART NAMES — `Hit(bd)` in the warp DSL — to the
    /// runtime channel slot for that part's brush.
    pub name_to_brush_idx: HashMap<String, u32>,
    /// Inline-anonymous warp name → the audio def its `| warp { … }` was
    /// found inside. Used by `fit_length::apply_implicit_fit_length` to give
    /// each inline warp's top-level Seq an implicit cycle length equal to
    /// `length(<attached_def>)` — composers don't have to write `FitLength`
    /// (or `ModBy`) explicitly.
    pub inline_warp_attachments: HashMap<String, String>,
    /// Warp name → `(body_start, body_end)` byte range in the ORIGINAL
    /// source. The body is the text between `{` and the matching `}` in
    /// either form (`warp NAME = { body }` or `audio | warp { body }`).
    /// The promote-pass uses this to scope its source-scan to just the
    /// warp bodies — otherwise it'd see literals from `{ f: 311.127, ... }`
    /// or audio note tuples and desync against the warp AST.
    pub warp_body_spans: HashMap<String, (usize, usize)>,
}

#[derive(Debug)]
pub enum PreprocessError {
    UnbalancedBraces { start: usize },
    BadName { start: usize },
    Parse { name: String, err: ParseError },
}

impl PreprocessError {
    /// If the variant carries a parse error, render it with full source
    /// context (`dsl_parse_error::DslParseError::display`). For structural
    /// errors (unbalanced braces etc.) prints a one-liner. `quiet`
    /// suppresses output entirely. Call this at the user-facing error
    /// boundary so failures look like weresocool's, not Rust's.
    pub fn display(&self, quiet: bool) {
        if quiet { return; }
        match self {
            PreprocessError::Parse { name, err } => {
                eprintln!("[warp] inside `warp {} = {{ … }}`:", name);
                err.display(false);
            }
            PreprocessError::UnbalancedBraces { start } =>
                eprintln!("[warp] unbalanced braces starting at byte {}", start),
            PreprocessError::BadName { start } =>
                eprintln!("[warp] invalid warp name at byte {}", start),
        }
    }
}

/// Scan the source for `warp <name> = { ... }` blocks, pull them out, AND
/// scan top-level brush defs for `| warp <name>` chain ops — for each such
/// def, collect the color names it mentions (via `Color [...]`) and record
/// `color → warp_name`. The chain `| warp X` is also stripped so weresocool
/// doesn't see it.
pub fn extract_warps(source: &str) -> Result<Preprocessed, PreprocessError> {
    let bytes = source.as_bytes();
    // `out` is built as a Vec<u8> so byte-length matches the source exactly
    // — critical for byte-span correctness. The naive `String + push(bytes[i] as char)`
    // approach inflates non-ASCII (em dashes, arrows) when re-encoding, which
    // would break offset alignment for the promote-pass.
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut warps = Vec::new();
    let mut warp_blend_modes: HashMap<String, WarpBlendMode> = HashMap::new();
    // warp name → (body_start, body_end) in the ORIGINAL source — used by
    // the promote-pass to scope its source-literal scan to just the warp body.
    let mut warp_body_spans: HashMap<String, (usize, usize)> = HashMap::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // Look for the keyword `warp` at a word boundary.
        if matches_keyword(bytes, i, b"warp") {
            let block_start = i;
            let mut j = i + 4;
            j = skip_ws(bytes, j);

            let name_start = j;
            while j < bytes.len() && is_ident_byte(bytes[j]) { j += 1; }
            if j == name_start {
                out.push(bytes[i]);
                i += 1;
                continue;
            }
            let name = source[name_start..j].to_string();

            let mut blend_mode = WarpBlendMode::Additive;
            let mut after = skip_ws(bytes, j);
            let mode_start = after;
            while after < bytes.len() && is_ident_byte(bytes[after]) { after += 1; }
            if after > mode_start {
                if let Some(m) = WarpBlendMode::from_keyword(&source[mode_start..after]) {
                    blend_mode = m;
                    j = after;
                }
            }

            let k = skip_ws(bytes, j);
            if k >= bytes.len() || bytes[k] != b'=' {
                out.push(bytes[i]); i += 1; continue;
            }
            let k = skip_ws(bytes, k + 1);

            if k >= bytes.len() || bytes[k] != b'{' {
                out.push(bytes[i]); i += 1; continue;
            }
            let body_start = k + 1;
            let body_end = match find_matching_brace(bytes, k) {
                Some(e) => e,
                None => return Err(PreprocessError::UnbalancedBraces { start: block_start }),
            };

            let body = &source[body_start..body_end];
            let pipeline = parse_pipeline(body)
                .map_err(|err| PreprocessError::Parse { name: name.clone(), err })?;
            if blend_mode != WarpBlendMode::Additive {
                warp_blend_modes.insert(name.clone(), blend_mode);
            }
            warp_body_spans.insert(name.clone(), (body_start, body_end));
            warps.push(WarpDef { name, pipeline });

            // Replace the skipped warp block with spaces (newlines preserved
            // for line-number-fidelity) so `out` stays byte-aligned with the
            // original source. weresocool sees a stretch of whitespace where
            // the warp def used to be — semantically equivalent to elision,
            // structurally byte-identical to source. This means inline-warp
            // body spans recorded later (against `out`-coords) ARE source
            // coords, so the promote-pass can scope correctly.
            let block_end = body_end + 1;
            for p in i..block_end {
                out.push(if bytes[p] == b'\n' { b'\n' } else { b' ' });
            }
            i = block_end;
            if i < bytes.len() && bytes[i] == b'\n' {
                out.push(b'\n');
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    let warp_name_set: HashSet<String> = warps.iter().map(|w| w.name.clone()).collect();
    // `out` is now byte-aligned with the original source (warp blocks replaced
    // by spaces, multi-byte UTF-8 preserved verbatim), so positions returned
    // by strip_chain_warp_ops are valid original-source byte offsets.
    let out_str = String::from_utf8(out)
        .expect("warp stripper preserves UTF-8 byte sequences");
    let (second_pass_out, color_to_chain, inline_warps, inline_warp_attachments, inline_body_spans, name_to_brush_idx) =
        strip_chain_warp_ops(&out_str, &warp_name_set);
    // Merge inline body spans into the warp_body_spans map (which already
    // holds named-warp spans). After this, every WarpDef in `all_warps`
    // — named or inline — has an entry, and the promote-pass treats them
    // identically: scan-scoped to the body, literals become UserParams.
    for (name, span) in inline_body_spans {
        warp_body_spans.insert(name, span);
    }

    let mut all_warps = warps;
    all_warps.extend(inline_warps);

    Ok(Preprocessed {
        stripped: second_pass_out,
        warps: all_warps,
        color_to_chain,
        warp_blend_modes,
        inline_warp_attachments,
        warp_body_spans,
        name_to_brush_idx,
    })
}

/// Walk top-level let bindings looking for `| NAME` (NAME ∈ warp_names)
/// inside their bodies. Returns the source with those ops elided, plus a map
/// brush-index → warp_name. Brush index is the source-order position of the
/// def's `Color [...]` clause — that's what weresocool assigns as the
/// `BrushIdentifier` ("0", "1", ...).
struct DefInfo {
    body_start: usize,
    body_end: usize,
    has_color: bool,
    /// All `| <warp>` / `| warp { ... }` clauses in this def's body, in order
    /// of appearance. Multiple tags compose (cascade) — each one's output is
    /// the next one's scene input. Just like multiple `wgsl { ... }` blocks
    /// on a brush stack their vertex transforms.
    warp_tags: Vec<WarpTagSource>,
    refs: Vec<String>,
    stripped_body: String,
}

/// Two-pass: (1) find every top-level `name = { body }` and parse its tag,
/// references, etc. (2) BFS from each tagged def down through references,
/// tagging every leaf brush def encountered with that warp. So `| plasma`
/// on `main` propagates through `Seq [bar_basic, ...]` → `Overlay [kick_a, ...]`
/// → `Seq [bd, ...]` → bd's brush index.
fn strip_chain_warp_ops(
    source: &str,
    warp_names: &HashSet<String>,
) -> (String, HashMap<String, Vec<String>>, Vec<WarpDef>, HashMap<String, String>, HashMap<String, (usize, usize)>, HashMap<String, u32>) {
    let bytes = source.as_bytes();

    // Pass 1: collect all top-level let bindings (name → start of body, end of body).
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
                        // Placeholder for now; refs filled in below once we
                        // know all def names.
                        let (stripped_body, warp_tags, has_color) = scan_def_body(body, warp_names);
                        if !defs.contains_key(&name) {
                            def_order.push(name.clone());
                        }
                        defs.insert(name, DefInfo {
                            body_start: k + 1,
                            body_end,
                            has_color,
                            warp_tags,
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

    // Pass 2: now that we know all def names, scan each body for identifiers
    // that match — those are inter-def references.
    let def_names: HashSet<String> = defs.keys().cloned().collect();
    for info in defs.values_mut() {
        let body = &source[info.body_start..info.body_end];
        info.refs = collect_refs(body, &def_names);
    }

    // Pass 3: assign brush indices in source order — every def with a Color
    // clause gets the next sequential index. This matches weresocool's own
    // ordering, which is what `BrushIdentifier` ends up holding.
    let mut name_to_brush_idx: HashMap<String, u32> = HashMap::new();
    let mut next_idx = 0u32;
    for name in &def_order {
        if defs.get(name).map_or(false, |d| d.has_color) {
            name_to_brush_idx.insert(name.clone(), next_idx);
            next_idx += 1;
        }
    }

    // Pass 4 (pre): materialize every inline `| warp { ... }` block into a
    // named warp with a synthetic name. After this, each def has a `Vec<String>`
    // of resolved warp names — order = order they appeared in the chain.
    // Also record `__inline_<defname>_<i> → <defname>` so the cross-DSL
    // length env can apply an implicit FitLength to each inline warp's Seq
    // (cycle = length of the audio def the warp is attached to).
    let mut inline_warps: Vec<WarpDef> = Vec::new();
    let mut inline_attachments: HashMap<String, String> = HashMap::new();
    // Inline warp anon-name → (body_start, body_end) in the ORIGINAL
    // source (NOT in the def-body slice). Lifted from the per-tag
    // body_span (which is def-body-local) by adding the def's body_start.
    // Lets the promote-pass treat inline warps the same as named warps —
    // their literals get UserParam slots and the UI sees them.
    let mut inline_body_spans: HashMap<String, (usize, usize)> = HashMap::new();
    let mut names_for_def: HashMap<String, Vec<String>> = HashMap::new();
    for name in &def_order {
        let Some(info) = defs.get(name) else { continue };
        let mut resolved: Vec<String> = Vec::with_capacity(info.warp_tags.len());
        for (i, tag) in info.warp_tags.iter().enumerate() {
            resolved.push(match tag {
                WarpTagSource::Named(n) => n.clone(),
                WarpTagSource::Inline { pipeline, body_span } => {
                    let anon = format!("__inline_{}_{}", name, i);
                    inline_warps.push(WarpDef {
                        name: anon.clone(),
                        pipeline: pipeline.clone(),
                    });
                    inline_attachments.insert(anon.clone(), name.clone());
                    let global = (
                        info.body_start + body_span.0,
                        info.body_start + body_span.1,
                    );
                    inline_body_spans.insert(anon.clone(), global);
                    anon
                }
            });
        }
        names_for_def.insert(name.clone(), resolved);
    }

    // Pass 5: build each leaf brush's CASCADE CHAIN.
    //
    // The chain is built as: leaf's own warps first, then for every
    // composition def (no Color) whose ref-graph transitively contains this
    // leaf, its warps appended in source order. So `main | plasma | sphere`
    // on a leaf with `| pulse` becomes the chain [pulse, plasma, sphere]:
    // pulse runs first on the brush, plasma takes pulse's output, sphere
    // takes plasma's output. Mirrors how multiple `wgsl { ... }` blocks
    // compose on a brush — each one is a stage.
    let mut index_to_chain: HashMap<String, Vec<String>> = HashMap::new();
    for leaf_name in &def_order {
        let Some(leaf_info) = defs.get(leaf_name) else { continue };
        if !leaf_info.has_color { continue }
        let Some(&idx) = name_to_brush_idx.get(leaf_name) else { continue };

        let mut chain: Vec<String> = names_for_def.get(leaf_name).cloned().unwrap_or_default();

        // Append warps from every composition def that transitively reaches
        // this leaf (in source order, so ordering is deterministic).
        for comp_name in &def_order {
            if comp_name == leaf_name { continue }
            let Some(comp_info) = defs.get(comp_name) else { continue };
            if comp_info.has_color { continue }
            let comp_names = names_for_def.get(comp_name);
            if comp_names.map_or(true, |v| v.is_empty()) { continue }
            if reaches(comp_name, leaf_name, &defs) {
                chain.extend(comp_names.unwrap().iter().cloned());
            }
        }

        if !chain.is_empty() {
            index_to_chain.insert(idx.to_string(), chain);
        }
    }

    // Pass 5: reassemble the source — emit non-def regions as-is, replace
    // each def's body with its stripped version.
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    // Sort defs by source position.
    let mut by_pos: Vec<(&String, &DefInfo)> = defs.iter().collect();
    by_pos.sort_by_key(|(_, d)| d.body_start);
    for (_name, info) in &by_pos {
        // emit chars from cursor up to (body_start - 1) which is the `{`
        out.push_str(&source[cursor..info.body_start]); // includes `{`
        out.push_str(&info.stripped_body);
        out.push('}');
        cursor = info.body_end + 1;
    }
    out.push_str(&source[cursor..]);

    (out, index_to_chain, inline_warps, inline_attachments, inline_body_spans, name_to_brush_idx)
}

/// True if `from` transitively references `to` through the def-ref graph.
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

/// Scan a def's body for warp clauses. Returns (stripped_body, tags_in_order, has_color).
/// Tags are APPENDED, not overwritten — multiple `| warp { ... }` / `| NAME`
/// blocks in the same chain compose, each one becoming a stage in the cascade.
fn scan_def_body(body: &str, warp_names: &HashSet<String>) -> (String, Vec<WarpTagSource>, bool) {
    let scan = crate::dsl_extract::scan_def_body(body, b"warp", warp_names, /* allow_inline = */ true);
    let mut warp_tags = Vec::with_capacity(scan.tags.len());
    for t in scan.tags {
        match t {
            crate::dsl_extract::RefTag::Named(n) => warp_tags.push(WarpTagSource::Named(n)),
            // Inline `| warp { … }`. `body_span` stays relative to this def's
            // body slice (materialization lifts it to a global span via
            // `def.body_start`); `open+1 .. close` is the inner range.
            crate::dsl_extract::RefTag::Inline { body, open, close } => {
                if let Ok(pipe) = parse_pipeline(&body) {
                    warp_tags.push(WarpTagSource::Inline {
                        pipeline: pipe,
                        body_span: (open + 1, close),
                    });
                }
            }
        }
    }
    (scan.stripped, warp_tags, scan.has_color)
}

fn find_matching_bracket(bytes: &[u8], open: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open], b'[');
    let mut depth = 1i32;
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => { depth -= 1; if depth == 0 { return Some(i); } }
            _ => {}
        }
        i += 1;
    }
    None
}

