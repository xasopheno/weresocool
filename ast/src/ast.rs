use crate::datagen::Scale;
use crate::Term;
use num_rational::Rational64;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FunDef {
    pub name: String,
    pub vars: Vec<String>,
    pub term: Box<Term>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Op {
    AsIs,
    Id(String),
    Tag(String),
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, Eq)]
/// Oscillator Type
pub enum OscType {
    None,
    Sine { pow: Option<Rational64> },
    Triangle { pow: Option<Rational64> },
    Square { width: Option<Rational64> },
    Noise,
    Saw,
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
