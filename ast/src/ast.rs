use crate::operations::{helpers::handle_id_error, NormalForm};
use indexmap::IndexMap;
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Term {
    Op(Op),
    Nf(NormalForm),
    FunDef(FunDef),
    Lop(ListOp),
    Lnf(ListNf),
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct ListNf(pub Vec<NormalForm>);

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    List(Vec<Term>),
    IndexedList { terms: Vec<Term>, indices: Indices },
    IndexedNamedList { name: String, indices: Indices },
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Indices {
    IndexList(IndexList),
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct IndexList {
    pub indices: Vec<Index>,
}

impl IndexList {
    pub fn new(indices: Vec<Index>) -> Self {
        let mut result = vec![];
        for index in indices {
            match index {
                Index::Index(index) => result.push(Index::Index(index)),
                Index::Random(index, seed) => result.push(Index::Random(index, seed)),
                Index::IndexAndTerm { index, term } => {
                    result.push(Index::IndexAndTerm { index, term })
                }
            }
        }
        Self { indices: result }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Index {
    Index(i64),
    Random(i64, Option<i64>),
    IndexAndTerm { index: i64, term: Term },
}

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

pub fn is_choice_op(term: Term, defs: &Defs) -> bool {
    match term {
        Term::FunDef(_) => unimplemented!(),
        Term::Lop(_) => unimplemented!(),
        Term::Lnf(_) => unimplemented!(),
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
            | Op::FunctionCall { .. }
            | Op::Silence { .. } => false,

            Op::Choice { .. } => true,

            Op::Id(id) => is_choice_op(handle_id_error(id, defs), defs),
            Op::WithLengthRatioOf { .. } => false,

            Op::Focus {
                op_to_apply, main, ..
            } => is_choice_op(*op_to_apply, defs) | is_choice_op(*main, defs),

            Op::Sequence { operations }
            | Op::ModulateBy { operations }
            | Op::Compose { operations }
            | Op::Overlay { operations } => {
                for operation in operations {
                    if is_choice_op(operation, defs) {
                        return true;
                    }
                }
                false
            }
        },
    }
}
