pub mod normalize_listop;
pub mod substitute_list;

use crate::{NormalForm, Term};

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct ListNf(pub Vec<NormalForm>);

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    Const(Vec<Term>),
    //NamedList { name: String, indices: Indices },
    //IndexedList { list: Box<ListOp>, indices: Indices },
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
                } //Index::RandomAndTerm { n, seed, term } => unimplemented!(),
            }
        }
        Self { indices: result }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Index {
    Index(i64),
    Random(i64, Option<i64>),
    //RandomAndTerm {
    //n: i64,
    //seed: Option<i64>,
    //term: Term,
    //},
    IndexAndTerm { index: i64, term: Term },
}
