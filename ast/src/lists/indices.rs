use crate::Term;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct IndexVector {
    pub index: usize,
    pub index_terms: Vec<Term>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Indices(pub Vec<Index>);

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Index {
    Const { index: Vec<i64> },
    Random { n: i64, seed: i64 },
    IndexAndTerm { index: Box<Index>, term: Term },
}

impl Indices {
    pub fn get_indices_and_terms(&self, len_list: usize) -> Vec<IndexVector> {
        let mut result = vec![];

        self.0.iter().for_each(|index| {
            result.append(&mut index.get_indices_and_terms(len_list));
        });
        result
    }
}

impl Index {
    pub fn get_indices_and_terms(&self, len_list: usize) -> Vec<IndexVector> {
        match self {
            Index::Const { index } => index
                .iter()
                .map(|i| IndexVector {
                    index: *i as usize,
                    index_terms: vec![],
                })
                .collect(),
            Index::Random { n, seed } => {
                let mut rng: StdRng = SeedableRng::seed_from_u64(*seed as u64);
                let mut result = vec![];
                for _ in 0..*n {
                    let n: usize = rng.gen_range(0, len_list);
                    result.push(IndexVector {
                        index: n,
                        index_terms: vec![],
                    });
                }

                result
            }
            Index::IndexAndTerm { index, term } => {
                let mut result = index.get_indices_and_terms(len_list);
                result
                    .iter_mut()
                    .for_each(|index_vector| index_vector.index_terms.push(term.clone()));

                result
            }
        }
    }
}

