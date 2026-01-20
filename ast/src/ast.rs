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

/// Parameters for Kick drum synthesis
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct KickParams {
    // High-level meta-parameters (0-1 scale, can exceed 1 to push)
    /// Transient intensity: click, attack sharpness, transient curve (default: 0.5)
    pub punch: Option<Rational64>,
    /// Low-end weight: sub amount, saturation, longer decay (default: 0.5)
    pub body: Option<Rational64>,
    /// High-frequency presence: less harmonic damping, more click (default: 0.5)
    pub air: Option<Rational64>,
    /// Velocity sensitivity: how much gain affects timbre (default: 0.5)
    pub dynamics: Option<Rational64>,

    // Specific parameters (override meta-param mappings)
    /// Pitch envelope decay rate (default: 50)
    pub pitch_decay: Option<Rational64>,
    /// Starting pitch multiplier (default: 3, meaning 4x → 1x)
    pub pitch_range: Option<Rational64>,
    /// Amplitude decay rate (default: 8)
    pub amp_decay: Option<Rational64>,
    /// Sub-harmonic intensity (default: 0.2) - creates "chest thump"
    pub sub_amount: Option<Rational64>,
    /// Attack click intensity (default: 0.3)
    pub click_amount: Option<Rational64>,
    /// Click frequency multiplier (default: 8)
    pub click_freq: Option<Rational64>,
    /// Attack transient amount (default: 0.3)
    pub attack: Option<Rational64>,
    /// Decay multiplier for 2nd harmonic - higher = faster decay (default: 1.5)
    pub harmonic_damping: Option<Rational64>,
    /// Soft saturation amount during decay (default: 0.3)
    pub saturation: Option<Rational64>,
    /// How much velocity affects spectrum (default: 0.5)
    pub velocity_tilt: Option<Rational64>,
    /// Attack spike shape curve (1=linear, 2=squared) (default: 2.0)
    pub transient_curve: Option<Rational64>,
}

/// Parameters for Snare drum synthesis
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct SnareParams {
    // High-level meta-parameters (0-1 scale, can exceed 1 to push)
    /// Transient intensity: attack sharpness, shell pitch range (default: 0.5)
    pub punch: Option<Rational64>,
    /// Resonance weight: saturation, longer shell decay (default: 0.5)
    pub body: Option<Rational64>,
    /// High-frequency presence: more wire, brightness (default: 0.5)
    pub air: Option<Rational64>,
    /// Velocity sensitivity: how much gain affects timbre (default: 0.5)
    pub dynamics: Option<Rational64>,

    // Specific parameters (override meta-param mappings)
    /// Tone pitch decay rate (default: 80)
    pub pitch_decay: Option<Rational64>,
    /// Tone pitch range (default: 2)
    pub pitch_range: Option<Rational64>,
    /// Shell amplitude decay rate (default: 12)
    pub shell_decay: Option<Rational64>,
    /// Wire amplitude decay rate (default: 20)
    pub wire_decay: Option<Rational64>,
    /// Wire mix ratio 0=all shell, 1=all wire (default: 0.6)
    pub wire_mix: Option<Rational64>,
    /// Bottom head frequency ratio (default: 1.8)
    pub shell_tune: Option<Rational64>,
    /// Attack transient amount (default: 0.4)
    pub attack: Option<Rational64>,
    /// Shell pitch envelope decay rate (default: 30)
    pub shell_pitch_decay: Option<Rational64>,
    /// Shell pitch envelope range (default: 0.3)
    pub shell_pitch_range: Option<Rational64>,
    /// Top/bottom head decay ratio (default: 1.6)
    pub head_damping_ratio: Option<Rational64>,
    /// Soft saturation amount (default: 0.2)
    pub saturation: Option<Rational64>,
    /// How much velocity affects spectrum (default: 0.5)
    pub velocity_tilt: Option<Rational64>,
    // Backwards compatibility aliases (deprecated)
    pub tone_decay: Option<Rational64>,
    pub noise_decay: Option<Rational64>,
    pub noise_mix: Option<Rational64>,
}

/// Parameters for HiHat synthesis
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct HiHatParams {
    // High-level meta-parameters (0-1 scale, can exceed 1 to push)
    /// Transient intensity: attack sharpness (default: 0.5)
    pub punch: Option<Rational64>,
    /// Resonance weight: saturation, sustain (default: 0.5)
    pub body: Option<Rational64>,
    /// High-frequency presence: brightness, shimmer (default: 0.5)
    pub air: Option<Rational64>,
    /// Velocity sensitivity: how much gain affects timbre (default: 0.5)
    pub dynamics: Option<Rational64>,

    // Specific parameters (override meta-param mappings)
    /// Amplitude decay rate (default: 25 closed, 4 open)
    pub decay: Option<Rational64>,
    /// Metallic shimmer frequency multiplier (default: 20)
    pub shimmer: Option<Rational64>,
    /// Scales high mode amplitudes for brightness control (default: 1.0)
    pub brightness: Option<Rational64>,
    /// Attack transient amount (default: 0.2)
    pub attack: Option<Rational64>,
    /// Pitch drop amount as energy dissipates (default: 0.02)
    pub pitch_drop: Option<Rational64>,
    /// Soft saturation amount (default: 0.1)
    pub saturation: Option<Rational64>,
    /// How much velocity affects spectrum (default: 0.3)
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
