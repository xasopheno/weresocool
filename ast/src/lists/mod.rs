pub mod normalize_listop;
pub mod substitute_list;

use crate::{NormalForm, Term};

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    ListNf(Vec<NormalForm>),
    ListTerm(ListTerm),
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListTerm {
    Const(Vec<Term>),
    Named(String),
    ListOpIndexed {
        list_op: Box<ListOp>,
        indices: Indices,
    },
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Indices(Vec<Index>);
#[derive(Clone, PartialEq, Debug, Hash)]
pub struct IndexVector {
    indices: Vec<i64>,
    terms: Vec<Term>,
}

impl Indices {
    pub fn new(indices: Vec<Index>) -> Self {
        let mut result = vec![];
        for index in indices {
            match index {
                Index::Const(index) => result.push(Index::Const(index)),
                Index::Random(index, seed) => result.push(Index::Random(index, seed)),
                Index::IndexAndTerm { index, term } => {
                    result.push(Index::IndexAndTerm { index, term })
                } //Index::RandomAndTerm { n, seed, term } => unimplemented!(),
            }
        }
        Self(result)
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Index {
    Const(i64),
    Random(i64, Option<i64>),
    IndexAndTerm { index: Box<Index>, term: Term },
    //RandomAndTerm {
    //n: i64,
    //seed: Option<i64>,
    //term: Term,
    //},
}
