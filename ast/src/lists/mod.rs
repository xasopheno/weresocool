pub mod indices;
pub mod normalize_listop;
pub mod substitute_list;

use crate::{GenOp, Term};

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    Const {
        terms: Vec<Term>,
    },
    Named {
        name: String,
    },
    ListOpIndexed {
        list_op: Box<ListOp>,
        indices: Indices,
        direction: Direction,
    },
    GenOp {
        gen: GenOp,
    },
    Concat {
        listops: Vec<ListOp>,
    },
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum Direction {
    Overlay,
    Sequence,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Indices(pub Vec<Index>);

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Index {
    /// @ [ i.. ]
    Const { indices: Vec<i64> },
    /// @ [ start:end ]
    Slice {
        start: Option<i64>,
        end: Option<i64>,
        skip: i64,
    },
    /// @ [ Random(n) ]
    Random { n: i64, seed: i64 },
    /// @ [ i | term ]
    IndexAndTerm { index: Box<Index>, term: Term },
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct TermVector {
    pub term: Term,
    pub index_terms: Vec<Term>,
}

impl From<Term> for TermVector {
    fn from(term: Term) -> Self {
        Self {
            term,
            index_terms: vec![],
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct IndexVector {
    pub index: usize,
    pub index_terms: Vec<Term>,
}
