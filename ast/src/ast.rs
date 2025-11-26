use crate::datagen::Scale;
use crate::Term;
use num_rational::Rational64;

/// Tracks the original syntax used for frequency/transpose operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum FmSyntax {
    #[default]
    Fm,
    Tm,
}

/// Tracks the original syntax used for frequency add operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum FaSyntax {
    #[default]
    Fa,
    Ta,
}

/// Tracks the original syntax used for Gain operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum GainSyntax {
    #[default]
    Gain,
    Gm,
}

/// Tracks the original syntax used for Length operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum LengthSyntax {
    #[default]
    Length,
    Lm,
}

/// Tracks the original syntax used for PanM operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum PanMSyntax {
    #[default]
    PanM,
    Pm,
}

/// Tracks the original syntax used for PanA operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum PanASyntax {
    #[default]
    PanA,
    Pa,
}

/// Tracks the original syntax used for Sequence operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum SeqSyntax {
    #[default]
    Seq,
    Sequence,
}

/// Tracks the original syntax used for Overlay operations
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum OverlaySyntax {
    #[default]
    Overlay,
    /// O[...] overtone shorthand
    O,
}

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
        #[allow(dead_code)]
        syntax: FmSyntax,
    },
    TransposeA {
        a: Rational64,
        #[allow(dead_code)]
        syntax: FaSyntax,
    },
    PanM {
        m: Rational64,
        #[allow(dead_code)]
        syntax: PanMSyntax,
    },
    PanA {
        a: Rational64,
        #[allow(dead_code)]
        syntax: PanASyntax,
    },
    Gain {
        m: Rational64,
        #[allow(dead_code)]
        syntax: GainSyntax,
    },
    Length {
        m: Rational64,
        #[allow(dead_code)]
        syntax: LengthSyntax,
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
        #[allow(dead_code)]
        syntax: SeqSyntax,
    },
    Overlay {
        operations: Vec<Term>,
        #[allow(dead_code)]
        syntax: OverlaySyntax,
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
