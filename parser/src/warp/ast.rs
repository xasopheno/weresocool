//! AST for the warp DSL.
//!
//! A warp is a `|`-piped chain that starts with a source (`Prev` or `Scene`)
//! and runs ops left-to-right. Numeric expressions can reference `Time`.
//! Compositors (`Max`, `Add`, etc.) take a parenthesized sub-pipeline.

// The expression sublanguage is shared across the visual DSLs — see
// `crate::dsl_expr` (DSL_STYLE Law 4). `WarpExpr` is an alias so warp keeps its
// spelling while the AST stays single-source. warp's grammar produces the
// `Lit`/`Time`/`Sin`/`Bin` subset; `warp::promote` injects `UserParam` (a
// literal promoted to a runtime-tweakable uniform slot — codegen emits
// `user_param(slot u)`; the value + source span live in the `SlotTable`).
pub use crate::dsl_expr::{BinOp, Expr as WarpExpr, HitRef, MathFn};

/// How a warp's final output joins the composited frame. Declared on the warp
/// definition: `warp shadow multiply = { ... }`. Only meaningful for the LAST
/// warp in a chain (the one whose output reaches the compositor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WarpBlendMode {
    /// Light adds (glows stack). Alpha ignored. The default.
    #[default]
    Additive,
    /// Premultiplied alpha-over: coverage hides what's beneath. Real opacity.
    Over,
    /// Darkens the layers beneath (vignettes, shadows).
    Multiply,
    /// Lightens, softer than additive (no clipping pile-up).
    Screen,
}

impl WarpBlendMode {
    pub fn from_keyword(s: &str) -> Option<Self> {
        match s {
            "additive" | "add" => Some(Self::Additive),
            "over"             => Some(Self::Over),
            "multiply" | "mult"=> Some(Self::Multiply),
            "screen"           => Some(Self::Screen),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Prev,
    Scene,
    /// Sine-wave pattern oscillator. `freq` = spatial frequency,
    /// `sync` = temporal scroll, `offset` = phase.
    Osc { freq: WarpExpr, sync: WarpExpr, offset: WarpExpr },
    /// Value noise. `scale` = spatial frequency, `offset` = time-drift speed.
    Noise { scale: WarpExpr, offset: WarpExpr },
    /// Linear gradient with direction `(x, y)` and phase `z`. Same shape
    /// as audio's per-brush `Gradient(x, y, z)` so composers can copy-paste
    /// the spelling across DSLs. `(x, y)` is the 2D direction in UV space;
    /// `z` shifts the gradient's zero-point along that direction.
    Gradient(WarpExpr, WarpExpr, WarpExpr),
    /// Constant color.
    Solid(WarpExpr, WarpExpr, WarpExpr),
}

/// Which channel(s) of the field a **substance verb** (MEDIUM.md M1) targets.
/// The `field2d` backend packs state into one RGBA16F: scalar fields are single
/// channels (`R/G/B/A`), a packed 2D velocity is `Gb`/`Rg`, a color is `Rgb`.
/// A field address for the substance verbs. `r/g/b/a/rg/gb/rgb` live in the
/// visible color buffer; `sx/sy/sz/sw/sxy/szw` live in the STATE texture — a
/// second full-precision feedback target that is never displayed, never
/// alpha-crushed, and needs no `Persist`: four true simulation channels
/// (Gray-Scott u/v, CA cells, height+velocity...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chan {
    R, G, B, A, Rg, Gb, Rgb, Sx, Sy, Sz, Sw, Sxy, Szw,
    /// An unresolved `state {}`-declared field name ("water", "organism").
    /// kintaro's resolve-fields pass rewrites it to a concrete variant
    /// before codegen; surviving to codegen is a bug (warn + treat as R).
    Named(String),
}

/// The velocity / force a substance verb reads: a scalar field's gradient
/// (`grad(height)`, a vec2 pointing uphill), its laplacian (`lap(height)`, a
/// scalar — the restoring force that makes waves oscillate rather than merely
/// spread), or a 2-channel field used directly as a vector (`gb`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSrc { Grad(Chan), Lap(Chan), Field(Chan) }

/// One op in a pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum WarpOp {
    // UV transforms
    Scale(WarpExpr),
    Rotate(WarpExpr),
    Scroll(WarpExpr, WarpExpr),
    Swirl(WarpExpr),
    Fold(WarpExpr),
    Kaleid(WarpExpr),
    ChromaShift(WarpExpr),
    /// Curl-noise displacement: `CurlFlow { amp: 0.012, freq: 9 }`.
    CurlFlow { amp: WarpExpr, freq: WarpExpr },
    // Color ops
    Decay(WarpExpr),
    Hue(WarpExpr),
    Gamma(WarpExpr),
    /// Soft-knee tonemap: `color = color / (1 + color * K)`. Caps highlights
    /// near `1/K` without clipping — essential for additive feedback chains.
    Knee(WarpExpr),
    /// Intensity / opacity for the additive+blend compositor. Multiplies RGB
    /// by the factor; coverage alpha follows. 1.0 = full, 0.5 = half-bright,
    /// 0.0 = invisible, >1.0 = boost.
    Gain(WarpExpr),
    Tint(WarpExpr, WarpExpr, WarpExpr),
    /// Map sampled-color luminance through a cosine palette with given offset.
    Palette(WarpExpr),
    /// 9-tap box blur of the current source at the current uv (radius in texels).
    Blur(WarpExpr),
    /// Radial darkening from frame edges — fades the corners toward
    /// black, leaving the centre at full brightness. Single-arg
    /// `strength` ∈ [0, 1]: 0 disables, 0.5 = soft fade, 1.0 = hard
    /// pinhole. Classic cinematic frame; gives a focal centre without
    /// needing additional geometry.
    Vignette(WarpExpr),
    /// Colour quantisation — snaps RGB channels to N evenly-spaced
    /// levels. `levels=4` produces cel-shaded / comic-book look;
    /// `levels=2` is near-binary high contrast; `levels=16+` is barely
    /// perceptible (smooth gradients survive). General-purpose
    /// stylisation primitive; pairs naturally with Bloom + Knee
    /// (quantise after tonemap → posterised neon).
    Posterize(WarpExpr),
    /// Sine displacement on UV — `Wave(freq, amp)` shifts UV.y by
    /// `sin(uv.x * freq) * amp` so horizontal lines undulate into
    /// vertical bands. Different from Modulate (which uses a separate
    /// source as displacement field) and Swirl (which rotates around
    /// origin): Wave is a directional shear. Stack two Waves with
    /// orthogonal phases for fabric-ripple.
    Wave { freq: WarpExpr, amp: WarpExpr },
    /// A radial wave centered on channel `ch`'s most recent note onset —
    /// the note is a stone dropped in the canvas. Rings expand from the
    /// hit position at `speed`, spaced by `freq`, displacing UV radially
    /// by `amp`, the whole effect gated by the channel's hit envelope
    /// (silent channel = no displacement). `Ripple(0, freq: 40, speed: 6,
    /// amp: 0.02)`. UV op: place BEFORE Decay/Add like Scale/Swirl.
    Ripple { ch: HitRef, freq: WarpExpr, speed: WarpExpr, amp: WarpExpr },
    /// A pool of colored light blooming AT channel `ch`'s most recent note
    /// onset, fading with its envelope — the canvas lights up where the
    /// music touches it. `Glow(0, r: 1, g: 0.8, b: 0.5, size: 0.25)`.
    /// Color op (additive): safe anywhere in the chain.
    Glow { ch: HitRef, r: WarpExpr, g: WarpExpr, b: WarpExpr, size: WarpExpr },
    /// A vortex centered on channel `ch`'s most recent note onset — the
    /// note STIRS the pool where it lands. Rotates the accumulated field
    /// around the hit position, falling off over `radius` (uv units) and
    /// gated by the hit envelope, so the liquid swirls on the hit and
    /// stills as it fades. Negative `strength` = counter-clockwise.
    /// `Stir(0, radius: 0.3, strength: 1.0)`. UV op: place before Decay/Add.
    Stir { ch: HitRef, radius: WarpExpr, strength: WarpExpr },
    /// THE generic hit-centered displacement primitive — Ripple, Bulge and
    /// Stir are idioms of it. Displaces sampling around the part's most
    /// recent onset: `radial` moves along the away-from-hit direction
    /// (positive = outward), `tangent` around it (positive = counter-
    /// clockwise). Both take full expressions; combine with Dist/HitAge/
    /// Hit to build any impact response:
    /// `Displace bd { radial: sin(Dist(bd) * 40 - HitAge(bd) * 6) * 0.02 * Hit(bd) }`.
    Displace { ch: HitRef, radial: WarpExpr, tangent: WarpExpr },
    /// A soft local swell at channel `ch`'s most recent onset: the region
    /// magnifies/distorts gently (falloff over `radius`), gated by the hit
    /// envelope, relaxing as it fades. `Bulge(0, radius: 0.22, amount: 0.05)`.
    Bulge { ch: HitRef, radius: WarpExpr, amount: WarpExpr, mirror: WarpExpr },
    /// THE COLLAPSE: treat the incoming source as a HEIGHT FIELD (luminance
    /// = elevation) and render it as a lit 3D relief flattened to 2D — the
    /// Lieberman move: form shown as smooth gradient, not as objects.
    /// `height` scales the terrain steepness; `lx`/`ly` aim the light.
    /// `Relief(height: 4, lx: 0.4, ly: 0.6)`.
    Relief { height: WarpExpr, lx: WarpExpr, ly: WarpExpr, ambient: WarpExpr },
    /// THE CLOTH: composite the incoming field as PIGMENT soaked into a
    /// woven canvas of color (r,g,b). The weave's tooth breaks the paint
    /// coverage (it catches the high threads first) and shows through the
    /// stain; unpainted regions are the bare linen. Multiplicative — the
    /// first light-ground, paint-darkens compositing in the warp.
    /// `Stain(r: 0.93, g: 0.90, b: 0.84, strength: 2.2)`.
    Stain { r: WarpExpr, g: WarpExpr, b: WarpExpr, strength: WarpExpr, opacity: WarpExpr },
    /// A REAL watercolor simulation step, run every frame in the feedback:
    /// the state buffer holds pigment (rgb) and water (alpha). Water
    /// diffuses and dries; pigment rides the water — it migrates toward
    /// the drying edge (edge blooms), bleeds while wet, settles into the
    /// paper's valleys (granulation), and STOPS where dry. Fresh marks
    /// deposit pigment + water, so strokes into wet regions bleed together
    /// and strokes onto dry paper stay crisp. Must live in a Prev-sourced
    /// stage with `| Opaque` (water needs the alpha channel).
    /// All knobs point the intuitive way (bigger = more of the word):
    /// `Watercolor { wetness: 0.9, drying: 0.006, bleed: 2.0,
    ///               deposit: 0.11, lift: 0.005 }`.
    Watercolor {
        wetness: WarpExpr,
        drying: WarpExpr,
        bleed: WarpExpr,
        deposit: WarpExpr,
        lift: WarpExpr,
    },

    // --- Substance verbs (MEDIUM.md M1) — the generic field-simulation core.
    // They read the feedback buffer (Prev) and mutate the working `color` field.
    /// Spread a field to its 5-tap neighbours: `F ← mix(F, avg, rate·gate)`.
    /// `gated` masks the rate by another field's value (0 = frozen, 1 = full) —
    /// e.g. watercolor pigment is mobile only where wet (`gated: a`).
    Diffuse { field: Chan, rate: WarpExpr, gated: Option<Chan> },
    /// Carry a field along a velocity by backward-trace resample:
    /// `F ← mix(F, sample(F, uv − V·amount), blend)`. `V` is a field's `grad`
    /// or a 2-ch field. `blend` (default 1) is how far to move toward the
    /// traced sample: 1 = full transport (wave height), <1 = a gentle pull
    /// (watercolor pigment edge-bloom, which must not teleport).
    Advect { field: Chan, by: FieldSrc, amount: WarpExpr, blend: WarpExpr },
    /// Accumulate a force into a (vec2) field: `F ← F + src·gain`. Pair with
    /// `Advect` for the wave equation (ripples) / buoyancy.
    Force { field: Chan, from: FieldSrc, gain: WarpExpr },
    /// Per-channel fade: `F ← F·(1 − by)`. The channel form `Decay a { by: … }`
    /// (the scalar-arg `Decay 0.95` stays the whole-rgb color op).
    DecayField { field: Chan, by: WarpExpr },
    /// Pointwise write: `F ← expr` (scalar, broadcast across F's components).
    /// The expr may read fields — `Set organism (clamp(organism + food *
    /// organism * organism - 0.11 * organism, 0.0, 1.0))` — which makes
    /// reaction terms, custom gates and init idioms expressible without Raw.
    /// SEQUENTIAL semantics: later ops (and later Sets) see the new value.
    Set { field: Chan, value: WarpExpr },
    /// Write the composition's marks (Scene) into a field — the one step every
    /// medium needs, previously hand-rolled in Raw. A multi-channel field takes
    /// Scene's matching channels; a single channel takes Scene's luminance
    /// (clamped 0‥1 — water/height/density semantics). `grain` modulates the
    /// deposit by static paper-noise (granulation/tooth, 0 = smooth); `cap`
    /// rescales multi-channel buildup that exceeds it (0 = uncapped).
    Deposit { field: Chan, gain: WarpExpr, grain: WarpExpr, cap: WarpExpr },
    /// Excitable travelling wave in a scalar field: rest (0) cells ignite to 1
    /// when a neighbouring cell holds a front (>0.85); ignited cells decay by
    /// `dry` each frame (the refractory tail) until they return to rest.
    /// Deposit into the field to FIRE a ring from every mark; the front then
    /// travels `step` pixels/frame outward. The medium's heartbeat primitive —
    /// pair with `Diffuse gated`/`Advect by grad`/`Flow gated` so the wave
    /// modulates the painting as it passes. Needs `Persist`.
    Propagate { field: Chan, step: WarpExpr, dry: WarpExpr },
    /// Constant-direction drift (gravity, wind, current): sample the field
    /// upstream by (x, y) uv/frame, optionally gated by another field so only
    /// e.g. WET pigment runs. `Flow rgb gated a { y: 0.005 }` = wet ink weeps
    /// downward.
    Flow { field: Chan, x: WarpExpr, y: WarpExpr, gated: Option<Chan> },
    /// Terminal marker: store the pipeline's alpha UNTOUCHED as hidden state.
    /// Unlike `Opaque` (alpha = mark presence) or the default epilogue
    /// (alpha = brightness), `Persist` lets a medium keep true internal state
    /// in the alpha channel across frames — cellular-automaton cells,
    /// excitation phase, age. Purely a codegen directive; emits nothing.
    Persist,
    /// Bloom: `Bloom { threshold: 0.5, strength: 0.6 }` — adds halo for bright pixels.
    Bloom { threshold: WarpExpr, strength: WarpExpr },
    /// Debug visualisation of the depth buffer (NDC depth stored in the
    /// scene texture's `.a` channel by the brush fragment). Outputs a
    /// turbo-style heatmap so you can verify depth is meaningful for the
    /// composition — near brushes show one colour, far brushes another.
    /// Replaces whatever colour was sampled this step. Drop it from the
    /// warp chain once you've confirmed depth makes sense.
    DepthVis,
    /// Decouple this chain's coverage from its brightness: alpha = "the
    /// scene has geometry here" instead of the default `max(rgb)`.
    ///
    /// Presence is `max(scene luminance, scene depth)` — the brush
    /// fragment stores NDC depth in the scene texture's `.a`, so even a
    /// pitch-black stamp registers. A chain containing `Opaque` skips the
    /// `alpha = max(rgb)` epilogue (which would otherwise overwrite it).
    ///
    /// This is what makes the `over` / `multiply` blend modes able to
    /// OCCLUDE: a dark silhouette layer (`warp trees over = { ... | Opaque }`)
    /// hides what's beneath it wherever its geometry sits. Without it,
    /// dark = transparent and additive is the only physics.
    Opaque,
    /// Raw WGSL — emitted verbatim at this point in the generated warp
    /// function. The string is inserted directly into the
    /// `fn warp_N(uv_in: vec2<f32>) -> vec4<f32>` body and can read/write
    /// the live `color: vec4<f32>` and `uv: vec2<f32>` locals, plus the
    /// `params.time` / `params.mic` uniforms and prelude helpers
    /// (`k_vnoise`, `k_curl`, `k_palette`, `k_blur9`, `k_bloom`).
    ///
    /// Syntax:  ``Raw `color.rgb = pow(color.rgb, vec3<f32>(0.8));` ``
    ///
    /// Backticks delimit; contents pass through unchanged. Use for
    /// custom per-pixel ops the structured vocabulary doesn't cover.
    Raw(String),
    /// Paint a static 2-band color gradient as the canvas. Useful for
    /// compositions whose subject IS a color field (Rothko, Hokusai, Munch).
    /// `Background(top_r, top_g, top_b, bot_r, bot_g, bot_b, split, soft)` —
    /// top color above `split` (uv.y), bottom color below; `soft` controls
    /// the smoothstep band-edge width (0 = hard line, 0.1 = soft fade).
    /// Replaces hand-rolled raw WGSL gradient blocks; runs after the rest
    /// of the warp chain so brushes pop above the field.
    Background {
        top_r: WarpExpr, top_g: WarpExpr, top_b: WarpExpr,
        bot_r: WarpExpr, bot_g: WarpExpr, bot_b: WarpExpr,
        split: WarpExpr, soft: WarpExpr,
    },
    /// Instantly wipe the field to black at a single composition time.
    /// `Clear(at: 30)` — at t=30s, set color to vec4(0). Hard cut. The
    /// next frame starts fresh; subsequent frames accumulate as normal.
    /// For recurring wipes, use `Clear(every: 30)` — fires at t=30, 60,
    /// 90… (modulo `every`). Set `offset` to phase-shift.
    Clear { at: Option<WarpExpr>, every: Option<WarpExpr>, offset: Option<WarpExpr> },
    /// Soft fade the field toward black over a window of composition time.
    /// `Fade { at: 30, dur: 1.0 }` — starting at t=30s, exponentially decay
    /// `Prev` faster than the steady-state `Decay` op for `dur` seconds,
    /// so the canvas darkens to near-zero by t=31s, then resumes normal
    /// accumulation. Composer-visible "soft reset between sections."
    /// `to` controls how dark we get (default 0.05 = 5% of original).
    /// `Fade { every: 30, dur: 1.0 }` for recurring fades aligned with
    /// `Repeat` boundaries — set `every` to the composition's length per
    /// iteration.
    Fade {
        at: Option<WarpExpr>,
        every: Option<WarpExpr>,
        offset: Option<WarpExpr>,
        dur: WarpExpr,
        to: WarpExpr,
    },
    /// Time-multiplexed sub-pipelines. Each phase runs for its `length`
    /// seconds (interpreted in top-level `l` units). The cycle repeats
    /// indefinitely, gated by `params.time % cycle_length`.
    /// Form: `Seq [pipeline | Lm m, pipeline | Lm n, ...]`. Items default
    /// to `Lm 1` if no explicit length.
    Seq { phases: Vec<SeqPhase> },
    /// `FitLength <name>` — proportionally rescales the *previous* Seq's
    /// phase Lm values so the cycle length equals `length(<name>)`
    /// (looked up at WGSL-emit time via the cross-DSL length env).
    /// Sequence ratio is preserved: `Seq [a | Lm 3, b | Lm 1] |
    /// FitLength thing` (with `length(thing) = 12`) becomes effective
    /// `Lm 9, Lm 3`. A FitLength immediately follows the Seq it
    /// rescales; the rewrite pass consumes the FitLength and updates
    /// the Seq's phases in place. If `<name>` isn't in the env (audio
    /// op unknown), FitLength is silently dropped — the Seq runs with
    /// its as-written Lm values.
    FitLength(String),
    /// Parallel sub-pipelines whose outputs combine additively. Like a
    /// repeated `Add(sub)` but takes a list. Form:
    /// `Overlay [pipeline_a, pipeline_b, ...]`.
    Overlay { layers: Vec<Vec<WarpOp>> },
    /// Identity / pass-through. `AsIs` in a warp pipeline emits no code —
    /// useful as a slot marker in a Seq: `Seq [AsIs | Lm 3, Fade(...) | Lm 1]`
    /// means "no warp change for 3s, then a fade."
    AsIs,
    /// Kill — sets `color = vec4<f32>(0)` so no contribution. In a Seq,
    /// `None | Lm 1` is a hard cut to black. Subsumes the dedicated
    /// `Clear` op. Source keyword `None`; AST variant `Mute` for parser
    /// hygiene (avoids shadowing `Option::None`).
    Mute,
    // Control flow
    /// Apply the inner sub-pipeline's ops N times to the current uv.
    /// Best with UV-only ops (Scale, Rotate, Fold, Kaleid, Scroll, Swirl, CurlFlow).
    Iterate(WarpExpr, Box<WarpPipeline>),
    // Compositors — take another pipeline as arg.
    Max(Box<WarpPipeline>),
    Add(Box<WarpPipeline>),
    Screen(Box<WarpPipeline>),
    /// Mix two pipelines: `Mix(p, amount)` — `amount` ∈ [0,1] picks the
    /// weighting between the current source and the sub-pipeline's output
    /// (0 = current only, 1 = sub only). Renamed from `Blend` to avoid
    /// colliding with audio's `Blend <color> <amount>` op which has a
    /// completely different signature and semantic ("tint a brush with a
    /// CSS color" vs "weighted mix of two visual streams").
    Mix(Box<WarpPipeline>, WarpExpr),
    /// Hydra-style displacement: run the sub-pipeline to a color, then use
    /// its `rg` channels (centered around 0.5) as per-pixel UV offset
    /// scaled by `amount`. The killer composition primitive — any stream
    /// can warp any other stream.
    Modulate(Box<WarpPipeline>, WarpExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarpPipeline {
    pub source: Source,
    pub ops: Vec<WarpOp>,
}

/// One phase of a warp `Seq`: a sub-pipeline plus its length in top-level
/// `l` units. Defaults to length 1 if no `| Lm N` is supplied.
///
/// `explicit_length` distinguishes "user wrote `| Lm 1`" (true) from
/// "grammar synthesised the default" (false) — the promote-pass needs
/// this to know whether `length` corresponds to a real numeric token in
/// the source file. Without it, every phase that omits Lm would push a
/// phantom literal into the source-walker's expectation and desync.
#[derive(Debug, Clone, PartialEq)]
pub struct SeqPhase {
    pub ops: Vec<WarpOp>,
    pub length: WarpExpr,
    pub explicit_length: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarpDef {
    pub name: String,
    /// `state { water: a, organism: sy }` — composer-chosen field names for
    /// channels. Empty when the block is absent. Names are usable anywhere a
    /// channel selector or a field expression appears in this def.
    pub state_names: Vec<(String, Chan)>,
    pub pipeline: WarpPipeline,
}
