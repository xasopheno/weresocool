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
/// Uses drum-specific spectrum controls (0-1 scale) plus specific parameter overrides
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct KickParams {
    // Spectrum controls (0-1 scale, can exceed 1 to push)
    /// Attack spectrum: soft/round (0) → hard/clicky (1). Controls click_amount, transient_curve
    pub attack: Option<Rational64>,
    /// Body spectrum: thin (0) → thick/subby (1). Controls sub_amount, hump
    pub body: Option<Rational64>,
    /// Tone spectrum: dark (0) → bright (1). Controls harmonic_damping, click_freq
    pub tone: Option<Rational64>,
    /// Length spectrum: tight (0) → boomy (1). Controls amp_decay
    pub length: Option<Rational64>,

    // Specific parameters (override spectrum mappings)
    /// Pitch envelope decay time in SECONDS (default: 0.03 = 30ms, 909-style click)
    pub pitch_decay: Option<Rational64>,
    /// Starting pitch multiplier (default: 3)
    pub pitch_range: Option<Rational64>,
    /// Amplitude decay time in SECONDS (default: 0.10-0.25s depending on length)
    pub amp_decay: Option<Rational64>,
    /// Sub-harmonic intensity (default: 0.5)
    pub sub_amount: Option<Rational64>,
    /// Attack click intensity (default: 0.25)
    pub click_amount: Option<Rational64>,
    /// Click frequency multiplier (default: 8)
    pub click_freq: Option<Rational64>,
    /// Decay multiplier for 2nd harmonic (default: 1.8)
    pub harmonic_damping: Option<Rational64>,
    /// Soft saturation amount (default: 0.2)
    pub saturation: Option<Rational64>,
    /// How much velocity affects spectrum (default: 0.5)
    pub velocity_tilt: Option<Rational64>,
    /// Attack spike shape curve (default: 2.0)
    pub transient_curve: Option<Rational64>,
    /// Tuning multiplier for base frequency (default: 1.0)
    pub tune: Option<Rational64>,
    /// Filter resonance Q (default: 1.0)
    pub resonance: Option<Rational64>,
    /// Envelope hump - sustain level before decay (default: 0.8)
    pub hump: Option<Rational64>,
}

/// Parameters for Snare drum synthesis
/// Uses drum-specific spectrum controls (0-1 scale) plus specific parameter overrides
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct SnareParams {
    // Spectrum controls (0-1 scale, can exceed 1 to push)
    /// Attack spectrum: soft (0) → cracking (1). Controls attack, crack, shell_pitch_range
    pub attack: Option<Rational64>,
    /// Wires spectrum: dry/woody (0) → sizzly (1). Controls wire_mix, wire_decay
    pub wires: Option<Rational64>,
    /// Tone spectrum: dark (0) → bright (1). Controls head_damping_ratio
    pub tone: Option<Rational64>,
    /// Length spectrum: tight (0) → ringy (1). Controls shell_decay
    pub length: Option<Rational64>,

    // Specific parameters (override spectrum mappings)
    /// Tone pitch decay rate (default: 80)
    pub pitch_decay: Option<Rational64>,
    /// Tone pitch range (default: 2)
    pub pitch_range: Option<Rational64>,
    /// Shell amplitude decay time in SECONDS (default: 0.10-0.20s, 909-style)
    pub shell_decay: Option<Rational64>,
    /// Wire amplitude decay time in SECONDS (default: 0.15-0.25s, 909-style)
    pub wire_decay: Option<Rational64>,
    /// Wire mix ratio 0=all shell, 1=all wire (default: 0.5)
    pub wire_mix: Option<Rational64>,
    /// Bottom head frequency ratio (default: 1.8)
    pub shell_tune: Option<Rational64>,
    /// Attack transient amount (default: 0.4)
    pub attack_amount: Option<Rational64>,
    /// Shell pitch envelope decay rate (default: 30)
    pub shell_pitch_decay: Option<Rational64>,
    /// Shell pitch envelope range (default: 0.3)
    pub shell_pitch_range: Option<Rational64>,
    /// Top/bottom head decay ratio (default: 1.5)
    pub head_damping_ratio: Option<Rational64>,
    /// Soft saturation amount (default: 0.15)
    pub saturation: Option<Rational64>,
    /// How much velocity affects spectrum (default: 0.5)
    pub velocity_tilt: Option<Rational64>,
    /// Filter resonance Q (default: 0.7)
    pub resonance: Option<Rational64>,
    /// Tonal crack amount (default: 0.2)
    pub crack: Option<Rational64>,
    /// Tuning multiplier on info.frequency for shell fundamental (default: 1.0)
    pub tune: Option<Rational64>,
    // Backwards compatibility aliases (deprecated)
    pub tone_decay: Option<Rational64>,
    pub noise_decay: Option<Rational64>,
    pub noise_mix: Option<Rational64>,
}

/// Parameters for HiHat synthesis
/// Uses drum-specific spectrum controls (0-1 scale) plus specific parameter overrides
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, Default)]
pub struct HiHatParams {
    // Spectrum controls (0-1 scale, can exceed 1 to push)
    /// Attack spectrum: soft (0) → clicky (1). Controls attack, pitch_drop
    pub attack: Option<Rational64>,
    /// Metal spectrum: dull (0) → shimmery (1). Controls shimmer, brightness
    pub metal: Option<Rational64>,
    /// Length spectrum: choked (0) → open (1). Controls decay_rate
    pub length: Option<Rational64>,

    // Specific parameters (override spectrum mappings)
    /// Amplitude decay rate - higher = faster decay (default: 20 closed, 5 open)
    pub decay_rate: Option<Rational64>,
    /// Metallic shimmer frequency multiplier (default: 25)
    pub shimmer: Option<Rational64>,
    /// Scales high mode amplitudes (default: 0.9)
    pub brightness: Option<Rational64>,
    /// Attack transient amount (default: 0.25)
    pub attack_amount: Option<Rational64>,
    /// Pitch drop amount (default: 0.015)
    pub pitch_drop: Option<Rational64>,
    /// Soft saturation amount (default: 0.08)
    pub saturation: Option<Rational64>,
    /// How much velocity affects spectrum (default: 0.4)
    pub velocity_tilt: Option<Rational64>,
    /// Filter resonance Q (default: 0.5)
    pub resonance: Option<Rational64>,
    /// Tuning multiplier on info.frequency for mode base (default: 1.0)
    pub tune: Option<Rational64>,
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
