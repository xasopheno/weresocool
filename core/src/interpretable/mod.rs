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
        let (filename, vec_string) = match &self {
            InputType::Filename(filename) => (filename, filename_to_vec_string(filename)?),
            InputType::Language(language) => (&"Language", language_to_vec_string(language)),
        };
        timing_print!("[Interpretable] Read file: {:?}", read_start.elapsed());

        let parse_start = timing_now!();
        // For `Filename` we hand the actual path through so a parse
        // error renders a clickable `file:line:col` header; for
        // `Language` (an in-memory snippet) there is no file to point
        // at, so we suppress the header by passing None.
        let source_name = match &self {
            InputType::Filename(filename) => Some(filename.to_string()),
            InputType::Language(_) => None,
        };
        let parsed_composition = parse_file(vec_string, None, working_path, source_name)?;
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
///   → warp/draw/surface extraction (strip blocks; ASTs go to the visual host)
///   → palette::extract_palette    (strip wrapper, hoist instrument defs)
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
    use weresocool_parser::{dsl_imports, dsl_params, draw, palette, surface_dsl, warp};

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

    let warp_pre = warp::extract_warps(&source)
        .map_err(|e| Error::with_msg(format!("warp: {e:?}")))?;
    let draw_pre = draw::extract_draws(&warp_pre.stripped)
        .map_err(|e| Error::with_msg(format!("draw: {e:?}")))?;
    let surface_pre = surface_dsl::extract_surfaces(&draw_pre.stripped)
        .map_err(|e| Error::with_msg(format!("surface: {e:?}")))?;
    let palette = palette::extract_palette(&surface_pre.stripped);

    Ok(AudioSource {
        source: palette.stripped,
        recordings,
    })
}
