//! LAYER extraction — the parsing half of the layers system (the runtime —
//! cameras, switching, stacking — lives in kintaro). Part of the language
//! front end: audio-only hosts run this via `preprocess_for_audio` so a
//! vanilla weresocool build parses (and silently strips) any piece that
//! uses layers.
//!
//! `layer [NAME =] { … }` bodies parse with the WARP grammar; `thing | NAME`
//! attachment pipes are recorded and blanked (they are DESUGARED into the
//! NormalForm downstream — each attached note's `ext.visual.attach`).

use crate::warp::WarpDef;

/// Extract every `layer [NAME =] { <warp pipeline> }` block/// Synthetic def name for the unnamed `layer { … }` fallback. Contains
/// `__` on purpose: the promote pass skips such names (no tweak sliders for
/// the anonymous block; named layers get sliders like any warp).
pub const FALLBACK_NAME: &str = "__bg";

#[derive(Debug, Clone)]
pub struct LayerExtract {
    /// Source with background defs AND `| name` attachments blanked to
    /// spaces (byte offsets preserved downstream).
    pub stripped: String,
    /// Backdrop defs as WARP defs, source order — index = backdrop id.
    pub warp_defs: Vec<WarpDef>,
    /// name → (start, end) byte span of the body in the (offset-stable)
    /// source, so the promote pass can bind tweak sliders.
    pub body_spans: std::collections::HashMap<String, (usize, usize)>,
    /// (audio def name, background name) attachment pairs.
    pub attachments: Vec<(String, String)>,
}


/// Extract every `layer [NAME =] { <warp pipeline> }` block (parsing the
/// body with the WARP grammar), then strip `| NAME` attachment pipes from the
/// remaining source, recording which audio def each binds to (the identifier
/// that starts the piped phrase: `thing1 | bg1` → thing1).
///
/// Errors (loudly, naming the def) when a body fails to parse as a warp
/// pipeline — old bare-WGSL bodies migrate by wrapping in `Raw { … }`.
pub fn extract_layers(source: &str) -> Result<LayerExtract, String> {
    let mut out = source.to_string().into_bytes();
    // All DETECTION runs on a `--`-comment-blanked shadow (offset-aligned
    // with `out`) so names inside socool comments never match. Bodies are
    // still sliced from the real bytes.
    let mut shadow = shadow_no_comments(source).into_bytes();
    let mut warp_defs: Vec<WarpDef> = Vec::new();
    let mut body_spans = std::collections::HashMap::new();

    // ── pass 0: the `background` keyword is DEAD — error loudly with the
    // migration. (It survived as vocabulary from before the layers
    // realization; a background is just a layer that sits behind.)
    {
        let sh = String::from_utf8_lossy(&shadow).into_owned();
        if let Some(kw) = find_keyword(&sh, "background", 0) {
            // `background_color:` in the header is fine (word boundary
            // already excludes it); a def-shaped use is not.
            let after = kw + "background".len();
            let rest = sh[after..].trim_start();
            if rest.starts_with('{') || rest.split('{').next().map(|pre| pre.trim().ends_with('=')).unwrap_or(false) {
                return Err(
                    "`background` is gone — it's just a layer. Write `layer [NAME =] { … }`; \
                     it sits behind the scene when attached (`def | NAME`) or placed before \
                     the audio voices in Overlay, above when placed after."
                        .to_string(),
                );
            }
        }
    }

    // ── pass 1: layer defs ──
    let mut from = 0;
    loop {
        let sh = String::from_utf8_lossy(&shadow).into_owned();
        let Some(kw) = find_keyword(&sh, "layer", from) else { break };
        let after_kw = kw + "layer".len();
        let Some(open_rel) = sh[after_kw..].find('{') else { break };
        let open = after_kw + open_rel;
        let between = sh[after_kw..open].trim();
        let name = between.strip_suffix('=').map(|n| n.trim().to_string());
        // `layer` followed by something that isn't `{` or `NAME = {`
        // isn't ours — skip past it.
        let name = match name {
            Some(n) if is_ident(&n) => n,
            Some(_) => {
                from = after_kw;
                continue;
            }
            None if between.is_empty() => FALLBACK_NAME.to_string(),
            None => {
                from = after_kw;
                continue;
            }
        };
        let Some(close) = match_brace(&shadow, open) else { break };
        // Body from the REAL bytes; convert `Raw { … }` → the lexer's
        // backtick form IN PLACE (same byte count — braces become ticks —
        // so all offsets stay stable).
        rawify(&mut out, open + 1, close);
        let real = String::from_utf8_lossy(&out).into_owned();
        let body = real[open + 1..close].to_string();
        // Try the full form (optional `state {}` + source) first; fall back
        // to the ops-only form (implicit `Prev`) so a body may start straight
        // at an op — `Raw { … } | Bloom …`.
        let (state_names, pipeline) =
            match crate::warp::parser_lalrpop::parse_pipeline_with_state_lalrpop(&body) {
                Ok(parsed) => parsed,
                Err(first_err) => match crate::warp::parser_lalrpop::parse_ops_only_lalrpop(&body)
                {
                    Ok(p) => (Vec::new(), p),
                    Err(_) => {
                        return Err(format!(
                            "layer `{}`: body is not a valid warp pipeline: {:?}\n\
                             (raw WGSL goes inside `Raw {{ … }}`)",
                            name, first_err
                        ))
                    }
                },
            };
        body_spans.insert(name.clone(), (open + 1, close));
        warp_defs.push(WarpDef {
            name: name.clone(),
            state_names,
            pipeline,
        });
        blank(&mut out, kw, close + 1);
        blank(&mut shadow, kw, close + 1);
        from = close + 1;
    }

    // ── pass 2: `| NAME [| Fade <secs>]` attachments ──
    let mut attachments: Vec<(String, String)> = Vec::new();
    if !warp_defs.is_empty() {
        let names: Vec<String> = warp_defs
            .iter()
            .filter(|d| d.name != FALLBACK_NAME)
            .map(|d| d.name.clone())
            .collect();
        let s = String::from_utf8_lossy(&shadow).into_owned();
        let b = s.as_bytes();
        for name in &names {
            let mut from = 0;
            while let Some(at) = find_keyword(&s, name, from) {
                from = at + name.len();
                // Must be a pipe target: previous non-ws char is `|`.
                let mut j = at;
                while j > 0 && (b[j - 1] as char).is_whitespace() {
                    j -= 1;
                }
                if j == 0 || b[j - 1] != b'|' {
                    continue;
                }
                let pipe = j - 1;
                // The attached def = first identifier of the piped phrase:
                // walk left to the phrase boundary, take its leading ident.
                let mut k = pipe;
                while k > 0 && !matches!(b[k - 1], b',' | b'[' | b'(' | b'{' | b'=' | b';') {
                    k -= 1;
                }
                let phrase = &s[k..pipe];
                let target = phrase
                    .split_whitespace()
                    .next()
                    .map(str::to_string)
                    .filter(|t| is_ident(t));
                match target {
                    Some(t) => attachments.push((t, name.clone())),
                    None => eprintln!(
                        "[background] `| {}` attachment: couldn't find the def it's piped \
                         onto — attachment ignored",
                        name
                    ),
                }
                // NOTE: `| Fade <ratio>` after the attachment is NOT handled
                // here — Fade is a real op of the AUDIO grammar (multiplies
                // the per-note fade field through normalization, like Lm).
                // It stays in the source; the timeline reads each note's
                // resolved fade from the renderables.
                blank(&mut out, pipe, at + name.len());
            }
        }
    }

    Ok(LayerExtract {
        stripped: String::from_utf8_lossy(&out).into_owned(),
        warp_defs,
        body_spans,
        attachments,
    })
}

/// Convert every `Raw { … }` inside `[from, to)` to the warp lexer's
/// ``Raw `…` `` form, IN PLACE. The open brace and its matching close brace
/// each become a backtick — byte count unchanged, offsets stable. (Arbitrary
/// WGSL can't pass through the socool lexer; backticks are its raw escape.)
fn rawify(out: &mut [u8], from: usize, to: usize) {
    let mut i = from;
    while i < to {
        let s = String::from_utf8_lossy(&out[..to]).into_owned();
        let Some(at) = find_keyword(&s, "Raw", i) else { break };
        if at >= to {
            break;
        }
        // Next non-ws char must be `{`.
        let mut j = at + 3;
        while j < to && (out[j] as char).is_whitespace() {
            j += 1;
        }
        if j >= to || out[j] != b'{' {
            i = at + 3;
            continue;
        }
        let Some(close) = match_brace(out, j) else { break };
        if close >= to {
            break;
        }
        out[j] = b'`';
        out[close] = b'`';
        i = close + 1;
    }
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Brace-match from `open`, skipping `//` line comments (a stray brace in a
/// WGSL comment can't desync the depth count).
fn match_brace(b: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
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

/// Blank [from, to) to spaces, keeping newlines (offset stability).
fn blank(out: &mut [u8], from: usize, to: usize) {
    for byte in out.iter_mut().take(to).skip(from) {
        if *byte != b'\n' {
            *byte = b' ';
        }
    }
}

/// Offset-preserving copy with `--` line comments blanked to spaces.
fn shadow_no_comments(source: &str) -> String {
    let mut out = source.as_bytes().to_vec();
    let mut i = 0;
    while i + 1 < out.len() {
        if out[i] == b'-' && out[i + 1] == b'-' {
            while i < out.len() && out[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Find `word` as a standalone token at/after `from`.
fn find_keyword(source: &str, word: &str, from: usize) -> Option<usize> {
    let mut from = from;
    while let Some(rel) = source[from..].find(word) {
        let at = from + rel;
        let before_ok = at == 0
            || !source.as_bytes()[at - 1].is_ascii_alphanumeric()
                && source.as_bytes()[at - 1] != b'_';
        let after = at + word.len();
        let after_ok = after >= source.len()
            || !source.as_bytes()[after].is_ascii_alphanumeric()
                && source.as_bytes()[after] != b'_';
        if before_ok && after_ok {
            return Some(at);
        }
        from = at + word.len();
    }
    None
}

