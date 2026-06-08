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
    Color(u64),
    Follow(crate::follow::types::Follow),
    AsIs,
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
    WGSL(u64),
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
    /// Annotate ops to be sent to MIDI channels
    Midi {
        channels: Vec<u8>,
    },
    //
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
    ColorMix {
        amount: Rational64,
    },
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
}

impl OscType {
    pub fn is_none(&self) -> bool {
        matches!(self, OscType::None)
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, OscType::None)
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
