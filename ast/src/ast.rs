extern crate indexmap;
extern crate num_rational;
use crate::operations::{helpers::handle_id_error, NormalForm};
use indexmap::IndexMap;
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Term {
    Op(Op),
    Nf(NormalForm),
}

pub type OpOrNfTable = IndexMap<String, Term>;

trait New<T> {
    fn new() -> T;
}

impl New<OpOrNfTable> for OpOrNfTable {
    fn new() -> OpOrNfTable {
        IndexMap::new()
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Op {
    AsIs,
    Id(String),
    Tag(String),
    //
    Fid(String),
    FunctionDef {
        name: String,
        vars: Vec<String>,
        op_or_nf: Box<Term>,
    },
    FunctionCall {
        name: String,
        args: Vec<Term>,
    },
    //
    Noise,
    Sine,
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
    Choice {
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
    Sine,
    Noise,
    Square,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, Eq)]
pub enum ASR {
    Short,
    Long,
}

pub fn is_choice_op(op_or_nf: Term, table: &OpOrNfTable) -> bool {
    match op_or_nf {
        Term::Nf(_) => false,
        Term::Op(op) => match op {
            Op::AsIs {}
            | Op::Sine {}
            | Op::Square {}
            | Op::Noise {}
            | Op::AD { .. }
            | Op::Portamento { .. }
            | Op::FInvert {}
            | Op::Reverse {}
            | Op::TransposeM { .. }
            | Op::TransposeA { .. }
            | Op::PanA { .. }
            | Op::PanM { .. }
            | Op::Gain { .. }
            | Op::Length { .. }
            | Op::Tag { .. }
            | Op::Fid { .. }
            | Op::FunctionDef { .. }
            | Op::FunctionCall { .. }
            | Op::Silence { .. } => false,

            Op::Choice { .. } => true,

            Op::Id(id) => is_choice_op(handle_id_error(id, table), table),
            Op::WithLengthRatioOf { .. } => false,

            Op::Focus {
                op_to_apply, main, ..
            } => is_choice_op(*op_to_apply, table) | is_choice_op(*main, table),

            Op::Sequence { operations }
            | Op::ModulateBy { operations }
            | Op::Compose { operations }
            | Op::Overlay { operations } => {
                for operation in operations {
                    if is_choice_op(operation, table) {
                        return true;
                    }
                }
                false
            }
        },
    }
}
