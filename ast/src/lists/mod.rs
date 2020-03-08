pub mod normalize_listop;
pub mod substitute_list;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

use crate::Term;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    Const(Vec<Term>),
    Named(String),
    ListOpIndexed {
        list_op: Box<ListOp>,
        indices: Indices,
    },
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Indices(Vec<Index>);

impl Indices {
    pub fn new(indices: Vec<Index>) -> Self {
        let mut result = vec![];
        for index in indices {
            match index {
                Index::Const { index, terms } => result.push(Index::Const { index, terms }),
                Index::Random { n, seed, terms } => result.push(Index::Random { n, seed, terms }),
                Index::IndexAndTerm { index, term } => {
                    result.push(Index::IndexAndTerm { index, term })
                }
            }
        }

        Self(result)
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Index {
    Const {
        index: Vec<i64>,
        terms: Vec<Term>,
    },
    Random {
        n: i64,
        seed: Option<i64>,
        terms: Vec<Term>,
    },
    IndexAndTerm {
        index: Box<Index>,
        term: Term,
    },
}

impl Indices {
    pub fn get_indices_and_terms(&self, len_list: usize) -> Vec<(Vec<usize>, Vec<Term>)> {
        let mut result = vec![];

        self.0.iter().for_each(|index| {
            result.push(index.get_indices_and_terms(len_list));
        });
        result
    }
}

impl Index {
    pub fn get_indices_and_terms(&self, len_list: usize) -> (Vec<usize>, Vec<Term>) {
        match self {
            Index::Const { index, terms } => {
                (index.iter().map(|i| *i as usize).collect(), terms.to_vec())
            }
            Index::Random { n, seed, terms } => {
                let mut rng: StdRng = match seed {
                    Some(s) => SeedableRng::seed_from_u64(*s as u64),
                    None => {
                        let mut rng = thread_rng();
                        let s = rng.gen::<u64>();
                        //println!("seed: {}", s);
                        SeedableRng::seed_from_u64(s as u64)
                    }
                };
                let mut indices = vec![];
                for _ in 0..*n {
                    let n: usize = rng.gen_range(0, len_list);
                    indices.push(n);
                }

                (indices, terms.to_vec())
            }
            Index::IndexAndTerm { index, term } => {
                let (indices, mut terms) = index.get_indices_and_terms(len_list);
                terms.push(term.clone());

                (indices, terms)
            }
        }
    }
}
