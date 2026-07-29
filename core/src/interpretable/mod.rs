use crate::generation::{parsed_to_render, RenderReturn, RenderType};
use std::path::PathBuf;
use weresocool_error::Error;
use weresocool_parser::parser::{filename_to_vec_string, language_to_vec_string, parse_file};
use weresocool_shared::{timing_now, timing_print};

pub enum InputType<'a> {
    Filename(&'a str),
    Language(&'a str),
}

pub trait Interpretable {
    fn make(
        &self,
        target: RenderType,
        working_path: Option<PathBuf>,
    ) -> Result<RenderReturn, Error>;
}

impl Interpretable for InputType<'_> {
    fn make(
        &self,
        target: RenderType,
        working_path: Option<PathBuf>,
    ) -> Result<RenderReturn, Error> {
        let read_start = timing_now!();
        // Read RAW source: the kintaro-DSL front end below wants whole text.
        let (filename, raw, base_dir, socool_path) = match &self {
            InputType::Filename(filename) => {
                let raw = std::fs::read_to_string(filename)
                    .map_err(|_| Error::with_msg(format!("File not found: {}", filename)))?;
                let path = std::path::PathBuf::from(filename);
                let base = path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                (*filename, raw, base, Some(path))
            }
            InputType::Language(language) => (
                "Language",
                language.to_string(),
                std::path::PathBuf::from("."),
                None,
            ),
        };
        timing_print!("[Interpretable] Read file: {:?}", read_start.elapsed());

        // THE front end (one pipeline for every host): on plain audio each
        // stage is a no-op passthrough; on a kintaro composition it inlines
        // `use` imports, injects DAW layers, expands parameterized defs,
        // and strips warp/draw/surface/palette blocks. parse_file no longer
        // carries its own duplicate strippers — this is the single place.
        let audio = preprocess_for_audio(&raw, &base_dir, socool_path.as_deref())?;
        let vec_string = language_to_vec_string(&audio.source);

        let parse_start = timing_now!();
        // For `Filename` we hand the actual path through so a parse
        // error renders a clickable `file:line:col` header; for
        // `Language` (an in-memory snippet) there is no file to point
        // at, so we suppress the header by passing None.
        let source_name = match &self {
            InputType::Filename(filename) => Some(filename.to_string()),
            InputType::Language(_) => None,
        };
        // Seed recorded layers (the `.socool.daw/` sidecar) so
        // `Perform("name")` ops resolve during normalization.
        let mut seed_defs = weresocool_ast::Defs::default();
        seed_defs.recordings = audio.recordings;
        let parsed_composition =
            parse_file(vec_string, Some(seed_defs), working_path, source_name)?;
        timing_print!("[Interpretable] parse_file: {:?}", parse_start.elapsed());

        let render_start = timing_now!();
        let result = parsed_to_render(filename, parsed_composition, target);
        timing_print!("[Interpretable] parsed_to_render: {:?}", render_start.elapsed());
        result
    }
}

/// Like `Interpretable::make`, but seeds a recordings registry so that
/// `Perform("name")` ops resolve during normalization. Unresolved names are
/// not an error — they come back via `Defs::pending_performs` on the returned
/// `RenderReturn::NfBasisAndTable(_, _, defs)`. Used by hosts (kintaro's DAW).
pub fn make_with_recordings(
    input: &InputType,
    target: RenderType,
    working_path: Option<PathBuf>,
    recordings: weresocool_ast::RecordingRegistry,
) -> Result<RenderReturn, Error> {
    let (filename, vec_string) = match input {
        InputType::Filename(filename) => (*filename, filename_to_vec_string(filename)?),
        InputType::Language(language) => ("Language", language_to_vec_string(language)),
    };
    let source_name = match input {
        InputType::Filename(filename) => Some(filename.to_string()),
        InputType::Language(_) => None,
    };

    // Seed the recordings into a fresh Defs and hand it to parse_file as the
    // starting definitions, so they're present during normalization.
    let mut seed_defs = weresocool_ast::Defs::default();
    seed_defs.recordings = recordings;

    let parsed_composition = parse_file(vec_string, Some(seed_defs), working_path, source_name)?;
    parsed_to_render(filename, parsed_composition, target)
}

// ─── kintaro-DSL compositions ───────────────────────────────────────

/// The audio half of a preprocessed kintaro composition: what the socool
/// grammar parses.
pub struct AudioSource {
    /// weresocool-ready source — `use` imports inlined, DAW layers injected,
    /// parameterized defs expanded, all kintaro DSL blocks stripped.
    pub source: String,
    /// Recordings that resolve `Perform("name")` ops during normalization.
    /// Hand to [`make_with_recordings`].
    pub recordings: weresocool_ast::RecordingRegistry,
}

/// Run the audio-relevant half of the kintaro-DSL front end
/// (`weresocool_parser::{dsl_imports, dsl_params, warp, draw, surface_dsl,
/// palette}` + the DAW sidecar from `crate::daw`):
///
/// ```text
/// raw source
///   → dsl_imports::resolve        (`use "std/warp"` → inlined text)
///   → daw::injection_for + apply  (recorded layers → __layer defs + registry)
///   → dsl_params::expand          (templates → specialized defs)
///   → warp/draw/surface/layer extraction (strip blocks; ASTs go to the visual host)
///   → palette::extract_palette    (strip wrapper, hoist instrument defs)
///   → layer stubs appended        (visual def names resolve as terms)
/// ```
///
/// `base_dir` anchors `use` imports (usually the composition's directory);
/// `socool_path` enables the `.socool.daw/` sidecar (recorded layers + named
/// recordings) and should be the on-disk path when there is one. Audio-only
/// hosts (the Logic AU plugin) chain this into [`make_with_recordings`];
/// kintaro runs the steps individually because it also wants the visual ASTs.
pub fn preprocess_for_audio(
    raw: &str,
    base_dir: &std::path::Path,
    socool_path: Option<&std::path::Path>,
) -> Result<AudioSource, Error> {
    use weresocool_parser::{dsl_compose, dsl_imports, dsl_let, dsl_params, draw, layer, light, palette, surface_dsl, warp};

    let source = dsl_imports::resolve(raw, base_dir)
        .map_err(|e| Error::with_msg(format!("import: {e}")))?;

    #[cfg(not(target_arch = "wasm32"))]
    let (source, recordings) = match socool_path {
        Some(path) => {
            let injection = crate::daw::injection_for(path);
            (
                crate::daw::apply_injection(&source, &injection),
                injection.registry,
            )
        }
        None => (source, weresocool_ast::RecordingRegistry::new()),
    };
    // wasm has no filesystem — no DAW sidecar to load.
    #[cfg(target_arch = "wasm32")]
    let (source, recordings) = {
        let _ = socool_path;
        (source, weresocool_ast::RecordingRegistry::new())
    };

    let source = dsl_params::expand(&source)
        .map_err(|e| Error::with_msg(format!("parameterized def: {e}")))?;

    // `let` preludes and draw composition, in the same order kintaro runs
    // them: params first (so a binding may use a template's argument), then
    // bindings, then composition. These lived only in kintaro's front end,
    // so every audio-only host — `print --wav` among them — choked on a
    // piece that used `let`, and the two front ends disagreed about what
    // the language IS. One pipeline, one answer.
    let source = dsl_let::expand(&source)
        .map_err(|e| Error::with_msg(format!("let binding: {e}")))?;
    let source = dsl_compose::expand(&source)
        .map_err(|e| Error::with_msg(format!("draw composition: {e}")))?;

    // Each extractor's `display(false)` prints the rich file:line:col report
    // (same renderer kintaro's extract path uses); the Error we bubble up
    // only needs to say which DSL failed.
    //
    // Lights first: they are leaves (nothing refers to a light from inside a
    // warp or a draw body) and the audio path has no use for them at all, so
    // getting them out of the way early keeps every later extractor unaware
    // that the keyword exists.
    let light_pre = light::extract_lights(&source).map_err(|e| {
        e.display(false);
        Error::with_msg("light block failed to parse (see report above)")
    })?;
    let source = light_pre.stripped;
    let warp_pre = warp::extract_warps(&source).map_err(|e| {
        e.display(false);
        Error::with_msg("warp block failed to parse (see report above)")
    })?;
    let draw_pre = draw::extract_draws(&warp_pre.stripped).map_err(|e| {
        e.display(false);
        Error::with_msg("draw block failed to parse (see report above)")
    })?;
    let surface_pre = surface_dsl::extract_surfaces(&draw_pre.stripped).map_err(|e| {
        e.display(false);
        Error::with_msg("surface block failed to parse (see report above)")
    })?;
    let layer_pre = layer::extract_layers(&surface_pre.stripped)
        .map_err(|e| Error::with_msg(format!("layer block failed to parse: {e}")))?;
    let palette = palette::extract_palette(&layer_pre.stripped);

    // LAYER STUBS: visual def names are first-class terms (`veil | FitLength
    // drums | Gm 1/2`). The stub's body is the real `Layer` op, so the audio
    // parse resolves them; their points are silenced BY TYPE downstream.
    // Appended (offset-stable) for every warp + layer def, same as kintaro's
    // visual path does — an audio-only host hears the identical piece.
    let mut source = palette.stripped;
    for name in warp_pre
        .warps
        .iter()
        .map(|w| &w.name)
        .chain(layer_pre.warp_defs.iter().map(|d| &d.name))
    {
        if !name.contains("__") {
            source.push_str(&format!("\n{} = {{ Layer(\"{}\") }}", name, name));
        }
    }

    Ok(AudioSource {
        source,
        recordings,
    })
}
