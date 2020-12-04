pub mod indices;
pub mod normalize_listop;
pub mod substitute_list;
use crate::NormalForm;
use num_rational::Rational64;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::Term;

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub idx: usize,
    pub axis: Axis,
    pub coefs: Vec<Rational64>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Axis {
    Fm,
    Fa,
    Gm,
    Lm,
    Pm,
    Pa,
}

impl FromStr for Axis {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "F*" => Ok(Axis::Fm),
            "F+" => Ok(Axis::Fa),
            "G*" => Ok(Axis::Gm),
            "L*" => Ok(Axis::Lm),
            "P*" => Ok(Axis::Pm),
            "P+" => Ok(Axis::Pa),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub state: NormalForm,
    pub idx: usize,
    pub list: ListOp,
    pub coefs: Vec<Coefs>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    Const(Vec<Term>),
    Named(String),
    ListOpIndexed {
        list_op: Box<ListOp>,
        indices: Indices,
    },
    Concat(Vec<ListOp>),
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
    term: Term,
    index_terms: Vec<Term>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct IndexVector {
    pub index: usize,
    pub index_terms: Vec<Term>,
}
