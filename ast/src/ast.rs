use crate::Term;
use indexmap::IndexMap;
use num_rational::Rational64;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FunDef {
    pub name: String,
    pub vars: Vec<String>,
    pub term: Box<Term>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Defs {
    pub terms: IndexMap<String, Term>,
    pub lists: IndexMap<String, Term>,
    pub generators: IndexMap<String, Term>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Op {
    AsIs,
    Id(String),
    Tag(String),
    //
    FunctionCall {
        name: String,
        args: Vec<Term>,
    },
    //
    Noise,
    Sine {
        pow: Option<Rational64>,
    },
    Square,
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
        m: Rational64,
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
        with_length_of: Box<Term>,
        main: Box<Term>,
    },

    Focus {
        name: String,
        main: Box<Term>,
        op_to_apply: Box<Term>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, Eq)]
pub enum OscType {
    None,
    Sine { pow: Option<Rational64> },
    Noise,
    Square,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, Eq)]
pub enum ASR {
    Short,
    Long,
}
