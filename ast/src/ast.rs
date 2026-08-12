use crate::datagen::Scale;
use crate::Term;
use num_rational::Rational64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq)]
pub struct MidiTarget {
    /// Zero-based MIDI channel (0..15)
    pub channel: u8,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FunDef {
    pub name: String,
    pub vars: Vec<String>,
    pub term: Box<Term>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Op {
    /// Extension ops (visual + midi) — see `ExtOp`.
    Ext(ExtOp),
    Follow(crate::follow::types::Follow),
    AsIs,
    /// Positional playback-start marker. Source keyword: `Start`.
    /// During normalization the handler does
    /// `input.start_at = Some(input.length_ratio)` — capturing the
    /// cumulative beat-time at the moment the marker appears in the
    /// op chain. Multiple `Start` markers overwrite each other, so
    /// the LAST one in the chain wins (i.e. the composer can move
    /// `| Start` around to change the playback start). No-op for
    /// length: `get_length_ratio` returns `1/1` (multiplicative
    /// identity), so `Start` doesn't lengthen the piece.
    Start,
    /// Kill — silence (semantic alias for `Gm 0` that also works in
    /// visual DSL contexts where `Gm` is meaningless). In ModBy/Seq
    /// chains, `None | Lm 3` means "no contribution for 3 base units."
    /// Source-level keyword: `None`. AST variant is `Mute` to avoid
    /// shadowing `Option::None` in the lalrpop-generated parser.
    Mute,
    Out,
    Id(String),
    Tag(String),
    /// Marker for Slice keepers - $name syntax
    Keeper(String),
    //
    //
    CSV1d {
        path: String,
        scale: Option<Rational64>,
    },
    CSV2d {
        path: String,
        scales: Vec<Scale>,
    },
    FromSound {
        path: String,
        voices: usize,
        fps: usize,
    },
    FromSoundYin {
        path: String,
        fps: usize,
    },
    /// Perform a named, pre-transcribed recording from the DAW recordings
    /// registry (see `Defs::recordings`). Resolves at normalization time:
    /// a hit applies the recording's NormalForm (like an inline `Id`); a
    /// miss is NOT an error — it renders silent and the name is collected
    /// in `Defs::pending_performs` so a host (kintaro) can pre-arm it.
    /// Source syntax: `Perform("name")`.
    Perform {
        name: String,
    },
    //
    FMOsc {
        defs: Vec<FmOscDef>,
    },
    Lowpass {
        hash: String,
        cutoff_frequency: Rational64,
        q_factor: Rational64,
    },
    Highpass {
        hash: String,
        cutoff_frequency: Rational64,
        q_factor: Rational64,
    },
    Bandpass {
        hash: String,
        cutoff_frequency: Rational64,
        q_factor: Rational64,
    },
    //
    FunctionCall {
        name: String,
        args: Vec<Term>,
    },
    Lambda {
        input_name: Option<String>,
        term: Box<Term>,
        scope: String,
    },
    //
    Noise,
    Saw,
    Sine {
        pow: Option<Rational64>,
    },
    Triangle {
        pow: Option<Rational64>,
    },
    Square {
        width: Option<Rational64>,
    },
    /// Kick drum - low frequency with pitch envelope
    Kick { params: Option<KickParams> },
    /// Snare drum - pitched component + noise burst
    Snare { params: Option<SnareParams> },
    /// Hi-hat (closed or open)
    HiHat { open: bool, params: Option<HiHatParams> },
    /// Clap - noise burst train + resonant tail
    Clap { params: Option<ClapParams> },
    /// Rimshot - short inharmonic resonator tock
    Rimshot { params: Option<RimshotParams> },
    /// Tom - pitched membrane, no wires
    Tom { params: Option<TomParams> },
    /// Ride cymbal - stick ping + bell + long wash
    Ride { params: Option<RideParams> },
    /// Crash cymbal - swelling broadband wash
    Crash { params: Option<CrashParams> },
    /// Shaker - dense granular particle noise
    Shaker { params: Option<ShakerParams> },
    /// Cowbell - two inharmonic square partials through a bandpass
    Cowbell { params: Option<CowbellParams> },

    #[allow(clippy::upper_case_acronyms)]
    AD {
        attack: Rational64,
        decay: Rational64,
        asr: ASR,
    },
    Portamento {
        m: Rational64,
    },
    //
    Reverse,
    FInvert,
    //
    Silence {
        m: Rational64,
    },
    TransposeM {
        m: Rational64,
    },
    TransposeA {
        a: Rational64,
    },
    PanM {
        m: Rational64,
    },
    PanA {
        a: Rational64,
    },
    Gain {
        m: Rational64,
    },
    Length {
        m: Rational64,
    },
    Reverb {
        m: Option<Rational64>,
    },
    Wavefolder {
        threshold: Rational64,
        stages: i64,
        input_gain: Rational64,
        output_gain: Rational64,
    },
    SoftClip {
        threshold: Rational64,
        input_gain: Rational64,
        output_gain: Rational64,
    },
    Overdrive {
        input_gain: Rational64,
        output_gain: Rational64,
    },
    Bitcrusher {
        bits: i64,
        input_gain: Rational64,
        output_gain: Rational64,
    },
    Tanh {
        input_gain: Rational64,
        output_gain: Rational64,
    },
    //
    Sequence {
        operations: Vec<Term>,
    },
    Overlay {
        operations: Vec<Term>,
    },
    Compose {
        operations: Vec<Term>,
    },
    ModulateBy {
        operations: Vec<Term>,
        /// Optional output mapping for reordering/transforming keepers
        /// e.g., ModBy [$a, $b] -> [b, a | Fm 2]
        output: Option<Vec<Term>>,
    },
    Choose {
        operations: Vec<Term>,
    },
    Repeat {
        operations: Vec<Term>,
        count: i64,
    },
    //
    //
    WithLengthRatioOf {
        main: Option<Box<Term>>,
        with_length_of: Box<Term>,
    },

    Focus {
        name: String,
        main: Box<Term>,
        op_to_apply: Box<Term>,
    },
}

/// EXTENSION OPS — every op that writes the point's `Ext` registry (kintaro
/// visual extension + the MIDI side-channel), grouped so the core `Op` enum
/// stays the sounds language. One delegating arm in normalize
/// (`ext::normalize_ext_op`) owns all of their normalization; none of them
/// affect length (`get_length_ratio` = 1) or synthesis.
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ExtOp {
    Color(u64),
    WGSL(u64),
    /// Visual crossfade length as a RATIO of the l-basis (synthesis-inert;
    /// consumed by the kintaro backdrop layer). Composes multiplicatively
    /// like Lm/Fm: `| Fade 1/2 | Fade 1/2` == `Fade 1/4`.
    Fade {
        m: Rational64,
    },
    /// A VISUAL LAYER CLIP as a first-class term: normalizes to a single
    /// point with the `layer` payload set (l=1 unit, g=1 → full opacity)
    /// and NO audio payload. All structural ops (Seq/Overlay/Lm/FitLength/
    /// Gm/Fade) compose over it exactly like a note — because it is one.
    Layer {
        name: String,
    },
    /// ATTACH a layer to sounding notes: the notes keep their audio and
    /// additionally DRIVE the named layer's visibility while they sound.
    /// `bd | pulse` is sugar for stamping this onto bd's points; the
    /// switching timeline is read straight off the NormalForm. The dual of
    /// `Layer` (clip = silent point IS the layer; attach = sounding point
    /// drives the layer).
    Attach {
        name: String,
    },
    /// LAYER PLACEMENT (term level, the transform family on layers):
    /// `Sm` scale (multiplies), `Xa`/`Ya` offset in screen fractions
    /// (adds), `Rz` rotation in turns (adds). Photoshop/Premiere position
    /// + scale, through normalization — schedulable per clip via Seq.
    LayerSm { m: Rational64 },
    LayerXa { a: Rational64 },
    LayerYa { a: Rational64 },
    LayerRz { a: Rational64 },
    /// Annotate ops to be sent to MIDI channels
    Midi {
        channels: Vec<u8>,
    },
    Hue {
        value: Rational64,
    },
    Saturation {
        value: Rational64,
    },
    Brightness {
        value: Rational64,
    },
    Vibrance {
        value: Rational64,
    },
    Gamma {
        value: Rational64,
    },
    ColorBlend {
        color_id: u64,
        amount: Rational64,
    },
    ColorAdd {
        color_id: u64,
    },
    ColorGradient {
        x: Rational64,
        y: Rational64,
        z: Rational64,
    },
    /// Visual layout: map this op's measured note-anchor extent on one
    /// axis onto the world-space band `[a, b]`. `axis` is 0=x, 1=y, 2=z.
    /// Sound-inert; consumed by kintaro. `FitX -1 1`, `FitY -9/10 1/10`,
    /// `FitZ -1/2 -1/2`.
    FitVis {
        axis: usize,
        a: Rational64,
        b: Rational64,
    },
    ColorMix {
        amount: Rational64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq)]
pub struct FmOscDef {
    pub fm: Rational64,
    pub depth: Rational64,
}

/// Parameters for Kick drum synthesis.
///
/// Top-level "spectrum" knobs (`attack` / `body` / `tone` / `length`) are
/// 0-1 macro controls that smear across several internal settings — the
/// fast way to dial in a sound. Specific named parameters below override
/// the spectrum mappings when you need precision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct KickParams {
    /// Named voicing this drum starts from (`Kick 808`). Preset values are
    /// the baseline; any params below override them. Valid names live in
    /// `crate::drum_presets::KICK_PRESETS`; the parser rejects unknown names.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// soft/round (0) → hard/clicky (1) — scales `click_amount` + transient.
    pub attack: Option<Rational64>,
    /// thin (0) → thick/fat (1) — scales saturation and shell ring.
    pub body: Option<Rational64>,
    /// dark (0) → bright (1) — scales `click_freq` and body cutoff range.
    pub tone: Option<Rational64>,
    /// tight (0) → boomy (1) — scales `amp_decay`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on `info.frequency` (default: 1.0).
    pub tune: Option<Rational64>,
    /// Two-stage pitch envelope total time in SECONDS (default: 0.045).
    pub pitch_decay: Option<Rational64>,
    /// Starting pitch multiplier — how high the sweep begins (default: 4.5).
    pub pitch_range: Option<Rational64>,
    /// Body amplitude decay time in SECONDS (default: 0.35-0.90, length-dependent).
    pub amp_decay: Option<Rational64>,
    /// Asymmetric soft-saturation drive (default: 0.40-0.70 from `body`).
    pub saturation: Option<Rational64>,
    /// Initial punch-hump depth, 4 ms peak (default: 0.50).
    pub hump: Option<Rational64>,
    /// Wooden shell-ring level — narrow BP at `f_base × 1.78 + 18 Hz` (default: 0.55).
    pub shell: Option<Rational64>,
    /// Karplus-Strong waveguide body blend (default: 0.85).
    pub ks_mix: Option<Rational64>,
    /// Filter-excited click amount (default: 0.10-0.45 from `attack`).
    pub click_amount: Option<Rational64>,
    /// Click bandpass center freq in Hz (default: 1.7-3.0 kHz from `tone`).
    pub click_freq: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre, not just level (default: 0.5).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Snare drum synthesis.
///
/// Top-level macros are `attack` / `wires` / `tone` / `length`; specific
/// overrides below. The snare is modeled as two head bands + 4-band wire
/// noise spectrum + Karplus-Strong waveguide body + filter-excited beater
/// + mid-band crack waveshaping.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct SnareParams {
    /// Named voicing this drum starts from (`Snare trap`). See
    /// `crate::drum_presets::SNARE_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// soft (0) → cracking (1) — scales beater click, crack, shell pitch range.
    pub attack: Option<Rational64>,
    /// dry/woody (0) → sizzly (1) — scales `wire_mix` and `wire_decay`.
    pub wires: Option<Rational64>,
    /// dark (0) → bright (1) — scales wire-band shift and mid-crack character.
    pub tone: Option<Rational64>,
    /// tight (0) → ringy (1) — scales `shell_decay`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on `info.frequency` (default: 1.0).
    pub tune: Option<Rational64>,
    /// Shell (head) decay time in SECONDS (default: 0.12-0.24, length-dependent).
    pub shell_decay: Option<Rational64>,
    /// Wire decay time in SECONDS (default: 0.14-0.30, wires-dependent).
    pub wire_decay: Option<Rational64>,
    /// Wire/shell balance, 0 = all shell, 1 = all wire (default: 0.35-0.70).
    pub wire_mix: Option<Rational64>,
    /// Second head-band ratio (default: 1.74, near first Bessel inharmonic).
    pub shell_tune: Option<Rational64>,
    /// Beater click amount — fed into a 3.5 kHz bandpass (default: 0.18-0.50).
    pub attack_amount: Option<Rational64>,
    /// Shell pitch-envelope decay rate (default: 40).
    pub shell_pitch_decay: Option<Rational64>,
    /// Shell pitch-envelope range (default: 0.15-0.50, attack-dependent).
    pub shell_pitch_range: Option<Rational64>,
    /// Bottom-head decay ratio relative to top (default: 1.10-1.80, tone-dependent).
    pub head_damping_ratio: Option<Rational64>,
    /// Broadband saturation drive (default: 0.20).
    pub saturation: Option<Rational64>,
    /// Tonal crack burst amount (default: 0.20-0.60 from `attack`).
    pub crack: Option<Rational64>,
    /// Mid-band crack waveshaper center freq in Hz (default: 3000).
    pub crack_freq: Option<Rational64>,
    /// Karplus-Strong waveguide body blend (default: 0.55).
    pub ks_mix: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.5).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for HiHat synthesis.
///
/// Built from 22 inharmonic modes (Bessel-like cymbal physics) with weak
/// nonlinear coupling, a 2-band noise air cascade with frequency-dependent
/// decay, and a brief pitched attack ping (stick-on-bell character).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct HiHatParams {
    /// Named voicing this drum starts from (`HiHat 909`, `OpenHat dust`).
    /// See `crate::drum_presets::HIHAT_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// soft (0) → clicky (1) — scales `attack_amount`, `pitch_drop`, ping.
    pub attack: Option<Rational64>,
    /// dull (0) → shimmery (1) — scales `shimmer` and `brightness`.
    pub metal: Option<Rational64>,
    /// choked (0) → open (1) — scales `decay_rate`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on `info.frequency` (default: 1.0).
    pub tune: Option<Rational64>,
    /// Modal/noise decay rate per second (default: 6-50 from `length`/open).
    pub decay_rate: Option<Rational64>,
    /// Fine multiplier on mode base after `tune` (default: 0.9-1.3).
    pub shimmer: Option<Rational64>,
    /// High-mode amplitude scaling (default: 0.5-1.1 from `metal`).
    pub brightness: Option<Rational64>,
    /// Attack stick-contact noise amount (default: 0.15-0.50).
    pub attack_amount: Option<Rational64>,
    /// Attack pitched-ping level — short ~2.5 kHz tone (default: 0.18).
    pub ping_amount: Option<Rational64>,
    /// Slight modal pitch droop during decay (default: 0.008-0.024).
    pub pitch_drop: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.4).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Clap synthesis.
///
/// Modeled as a short train of noise bursts (multiple hands striking a
/// few milliseconds apart) into resonant bandpasses, followed by a longer
/// noise tail — the classic analog clap architecture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct ClapParams {
    /// Named voicing (`Clap 808`). See `crate::drum_presets::CLAP_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// soft (0) → sharp/spitty (1) — burst envelope speed.
    pub attack: Option<Rational64>,
    /// tight (0) → wide (1) — spacing between the burst-train hits.
    pub spread: Option<Rational64>,
    /// dark (0) → bright (1) — bandpass centers and top-band level.
    pub tone: Option<Rational64>,
    /// dry (0) → roomy (1) — noise tail length.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on the bandpass centers (default: 1.0).
    pub tune: Option<Rational64>,
    /// Output saturation drive (default: preset).
    pub saturation: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.5).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Rimshot synthesis.
///
/// A stick striking rim and head together: two short inharmonic resonator
/// rings (woody/metallic "tock") plus a sharp click transient. Very short.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct RimshotParams {
    /// Named voicing (`Rimshot 808`). See `crate::drum_presets::RIMSHOT_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// woody (0) → metallic (1) — ring frequencies and balance.
    pub tone: Option<Rational64>,
    /// tick (0) → ring (1) — resonator decay time.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on the ring frequencies (default: 1.0).
    pub tune: Option<Rational64>,
    /// Click transient level (default: preset).
    pub attack_amount: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.5).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Tom synthesis.
///
/// A single struck membrane with no snare wires: six circular-membrane
/// Bessel modes over a Karplus-Strong waveguide body, a downward pitch
/// bend (the tom "voice"), a wooden shell resonance, and a mallet click.
/// Congas and bongos live here as presets — same physics, higher tuning
/// and a hand strike instead of a stick.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct TomParams {
    /// Named voicing (`Tom acoustic`). See `crate::drum_presets::TOM_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// soft/mallet (0) → hard stick (1) — scales the click and pitch bend.
    pub attack: Option<Rational64>,
    /// thin (0) → thick/resonant (1) — scales saturation and shell ring.
    pub body: Option<Rational64>,
    /// dark (0) → bright (1) — scales body cutoff range and mode tilt.
    pub tone: Option<Rational64>,
    /// tight (0) → long/booming (1) — scales `amp_decay`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on `info.frequency` (default: 1.0).
    pub tune: Option<Rational64>,
    /// Pitch-bend decay time in SECONDS (default: 0.10).
    pub pitch_decay: Option<Rational64>,
    /// Pitch-bend depth — starting multiple of the fundamental (default: 1.22).
    pub pitch_range: Option<Rational64>,
    /// Body amplitude decay time in SECONDS (default: 0.30-0.95, length-dependent).
    pub amp_decay: Option<Rational64>,
    /// Asymmetric soft-saturation drive (default: 0.25-0.55 from `body`).
    pub saturation: Option<Rational64>,
    /// Karplus-Strong waveguide body blend (default: 0.60).
    pub ks_mix: Option<Rational64>,
    /// Stick/mallet click level (default: 0.15-0.55 from `attack`).
    pub attack_amount: Option<Rational64>,
    /// Wooden shell-ring level (default: 0.30-0.45 from `body`).
    pub shell: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.6).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Ride cymbal synthesis.
///
/// A large plate: a defined stick *ping* (the articulation you actually
/// hear on a ride), a low group of long-decaying bell partials, and a
/// broad wash that accumulates under repeated hits. Shares the 22-mode
/// Bessel bank with the hi-hat but runs it bigger, longer, and with much
/// stronger nonlinear coupling.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct RideParams {
    /// Named voicing (`Ride bell`). See `crate::drum_presets::RIDE_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// washy (0) → sharply articulated stick ping (1).
    pub attack: Option<Rational64>,
    /// dull (0) → shimmery (1) — scales `shimmer` and `brightness`.
    pub metal: Option<Rational64>,
    /// short (0) → long ring (1) — scales `decay_rate`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on `info.frequency` (default: 1.0).
    pub tune: Option<Rational64>,
    /// Modal/noise decay rate per second (default: 1.6-4.0 from `length`).
    pub decay_rate: Option<Rational64>,
    /// Fine multiplier on the mode base after `tune` (default: 0.9-1.2).
    pub shimmer: Option<Rational64>,
    /// High-mode amplitude scaling (default: 0.45-1.05 from `metal`).
    pub brightness: Option<Rational64>,
    /// Stick ping level — the articulation (default: 0.30-0.85 from `attack`).
    pub attack_amount: Option<Rational64>,
    /// Bell-partial level — the low sustained "gong" under the ping (default: 0.35).
    pub bell_amount: Option<Rational64>,
    /// Broadband wash level (default: 0.30-0.55).
    pub wash: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.55).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Crash cymbal synthesis.
///
/// The opposite articulation to the ride: no defined ping, a *swell* into
/// a dense broadband explosion, then a long dark-tilting decay. The swell
/// is what stops a crash from reading as "a very loud hi-hat."
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct CrashParams {
    /// Named voicing (`Crash splash`). See `crate::drum_presets::CRASH_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// slow swell (0) → immediate explosion (1) — scales `swell`.
    pub attack: Option<Rational64>,
    /// dull (0) → shimmery (1) — scales `shimmer` and `brightness`.
    pub metal: Option<Rational64>,
    /// short (0) → very long (1) — scales `decay_rate`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on `info.frequency` (default: 1.0).
    pub tune: Option<Rational64>,
    /// Modal/noise decay rate per second (default: 0.9-2.6 from `length`).
    pub decay_rate: Option<Rational64>,
    /// Fine multiplier on the mode base after `tune` (default: 0.85-1.15).
    pub shimmer: Option<Rational64>,
    /// High-mode amplitude scaling (default: 0.55-1.20 from `metal`).
    pub brightness: Option<Rational64>,
    /// Swell rise time in SECONDS — 0 is an instant hit (default: 0.004-0.020).
    pub swell: Option<Rational64>,
    /// Broadband wash level (default: 0.55-0.85).
    pub wash: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.5).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Shaker synthesis.
///
/// Many small particles striking a shell within a few milliseconds. Built
/// as a jittered micro-burst train through a band-limited noise path, so
/// the "grain" of the sound is real granular density rather than a single
/// enveloped hiss. Tambourine, maraca, and cabasa are presets: the same
/// particle model with different shells and — for the tambourine — a pair
/// of ringing jingle bands on top.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct ShakerParams {
    /// Named voicing (`Shaker tambourine`). See `crate::drum_presets::SHAKER_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// soft (0) → sharp/snappy (1) — scales the onset burst weighting.
    pub attack: Option<Rational64>,
    /// dark/woody (0) → bright/metallic (1) — scales the band centers.
    pub tone: Option<Rational64>,
    /// tight (0) → long/spilling (1) — scales `decay`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on the noise band centers (default: 1.0).
    pub tune: Option<Rational64>,
    /// Overall decay time in SECONDS (default: 0.045-0.16, length-dependent).
    pub decay: Option<Rational64>,
    /// Particle density — micro-bursts per hit, 0 = one smooth burst (default: 0.55).
    pub density: Option<Rational64>,
    /// Top-band level (default: 0.35-0.95 from `tone`).
    pub brightness: Option<Rational64>,
    /// Metallic jingle-ring level — what makes a tambourine (default: 0.0).
    pub jingle: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.45).
    pub velocity_tilt: Option<Rational64>,
}

/// Parameters for Cowbell synthesis.
///
/// Two strongly inharmonic square-ish partials through a bandpass, plus a
/// mallet click — the 808 recipe, generalised. `acoustic` swaps the pure
/// pair for a clangier, denser set of struck-metal partials.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct CowbellParams {
    /// Named voicing (`Cowbell 808`). See `crate::drum_presets::COWBELL_PRESETS`.
    #[serde(default)]
    pub preset: Option<String>,

    // ── Spectrum macro knobs (0-1) ────────────────────────────────────
    /// dull (0) → clangy/bright (1) — partial ratio and bandpass center.
    pub tone: Option<Rational64>,
    /// tick (0) → long clank (1) — scales `decay`.
    pub length: Option<Rational64>,

    // ── Specific overrides ────────────────────────────────────────────
    /// Pitch multiplier on the partial frequencies (default: 1.0).
    pub tune: Option<Rational64>,
    /// Decay time in SECONDS (default: 0.10-0.40, length-dependent).
    pub decay: Option<Rational64>,
    /// Upper-partial frequency ratio — inharmonic by design (default: 1.48).
    pub ratio: Option<Rational64>,
    /// Mallet click level (default: 0.30-0.70 from `tone`).
    pub attack_amount: Option<Rational64>,
    /// How dramatically velocity (`Gm`) changes timbre (default: 0.5).
    pub velocity_tilt: Option<Rational64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq)]
/// Oscillator Type
pub enum OscType {
    None,
    Sine { pow: Option<Rational64> },
    Triangle { pow: Option<Rational64> },
    Square { width: Option<Rational64> },
    Noise,
    Saw,
    Fm { defs: Vec<FmOscDef> },
    /// Kick drum - low sine with pitch envelope
    Kick { params: Option<KickParams> },
    /// Snare drum - pitched sine + noise burst
    Snare { params: Option<SnareParams> },
    /// Hi-hat - noise with metallic shimmer
    HiHat { open: bool, params: Option<HiHatParams> },
    /// Clap - noise burst train + resonant tail
    Clap { params: Option<ClapParams> },
    /// Rimshot - short inharmonic resonator tock
    Rimshot { params: Option<RimshotParams> },
    /// Tom - struck membrane, no wires
    Tom { params: Option<TomParams> },
    /// Ride cymbal - stick ping + bell partials + wash
    Ride { params: Option<RideParams> },
    /// Crash cymbal - swelling broadband wash
    Crash { params: Option<CrashParams> },
    /// Shaker - granular particle noise (also tambourine/maraca/cabasa)
    Shaker { params: Option<ShakerParams> },
    /// Cowbell - two inharmonic partials through a bandpass
    Cowbell { params: Option<CowbellParams> },
}

impl OscType {
    pub fn is_none(&self) -> bool {
        matches!(self, OscType::None)
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, OscType::None)
    }

    /// Drum oscillators are one-shot, self-enveloping instruments — the
    /// renderer gives them different note-on/fade semantics than sustained
    /// tones (no outer attack ramp, persistent note clock, no crossfade
    /// between drum types).
    pub fn is_drum(&self) -> bool {
        matches!(
            self,
            OscType::Kick { .. }
                | OscType::Snare { .. }
                | OscType::HiHat { .. }
                | OscType::Clap { .. }
                | OscType::Rimshot { .. }
                | OscType::Tom { .. }
                | OscType::Ride { .. }
                | OscType::Crash { .. }
                | OscType::Shaker { .. }
                | OscType::Cowbell { .. }
        )
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, Eq)]
/// Attack/Sustain/Release Type
pub enum ASR {
    Short,
    Long,
}

/// Distortion effect type - stackable, applied after oscillator before filters
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, Eq)]
pub enum Distortion {
    Wavefolder {
        threshold: Rational64,
        stages: u8,
        input_gain: Rational64,
        output_gain: Rational64,
    },
    SoftClip {
        threshold: Rational64,
        input_gain: Rational64,
        output_gain: Rational64,
    },
    Overdrive {
        input_gain: Rational64,
        output_gain: Rational64,
    },
    Bitcrusher {
        bits: u8,
        input_gain: Rational64,
        output_gain: Rational64,
    },
    Tanh {
        input_gain: Rational64,
        output_gain: Rational64,
    },
}
