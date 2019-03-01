extern crate indexmap;
extern crate num_rational;
use crate::operations::{helpers::handle_id_error, NormalForm};
use indexmap::IndexMap;
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum OpOrNf {
    Op(Op),
    Nf(NormalForm),
}

pub type OpOrNfTable = IndexMap<String, OpOrNf>;

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
        op_or_nf: Box<OpOrNf>,
    },
    FunctionCall {
        name: String,
        args: Vec<OpOrNf>,
    },
    //
    Noise,
    Sine,
    Square,
    AD {
        attack: Rational64,
        decay: Rational64,
        length: usize,
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
        operations: Vec<OpOrNf>,
    },
    Overlay {
        operations: Vec<OpOrNf>,
    },
    Compose {
        operations: Vec<OpOrNf>,
    },
    Choice {
        operations: Vec<OpOrNf>,
    },
    ModulateBy {
        operations: Vec<OpOrNf>,
    },
    //
    WithLengthRatioOf {
        with_length_of: Box<OpOrNf>,
        main: Box<OpOrNf>,
    },

    Focus {
        name: String,
        main: Box<OpOrNf>,
        op_to_apply: Box<OpOrNf>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Hash)]
pub enum OscType {
    Sine,
    Noise,
    Square,
}

pub fn is_choice_op(op_or_nf: OpOrNf, table: &OpOrNfTable) -> bool {
    match op_or_nf {
        OpOrNf::Nf(_) => false,
        OpOrNf::Op(op) => match op {
            Op::AsIs {}
            | Op::Sine {}
            | Op::Square {}
            | Op::Noise {}
            | Op::AD { .. }
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
