use crate::{Index, IndexVector, Indices};
use rand::{rngs::StdRng, Rng, SeedableRng};

impl Indices {
    pub fn vectorize(&self, len_list: usize) -> Vec<IndexVector> {
        self.0
            .iter()
            .flat_map(|index| index.vectorize(len_list))
            .collect()
    }
}

impl Index {
    pub fn vectorize(&self, len_list: usize) -> Vec<IndexVector> {
        match self {
            Index::Const { indices } => indices
                .iter()
                .map(|i| IndexVector {
                    index: *i as usize,
                    index_terms: vec![],
                })
                .collect(),

            Index::Slice { start, end, skip } => {
                let a = match start {
                    Some(start) => {
                        if *start as usize > len_list as usize {
                            panic!(
                                "Start of slice {} is greater than length of list {}",
                                start, len_list
                            )
                        }
                        *start as usize
                    }
                    None => 0,
                };
                let b = match end {
                    Some(end) => {
                        if (*end as usize) > len_list as usize {
                            panic!(
                                "End of slice {} is greater than length of list {}",
                                end, len_list
                            )
                        }
                        *end as usize
                    }
                    None => (len_list - 1) as usize,
                };

                if a == b {
                    panic! {"start {} and end {} of slice are the same value", a, b};
                };

                if a < b {
                    (a..=b).collect::<Vec<usize>>()
                } else {
                    (b..=a).rev().collect::<Vec<usize>>()
                }
                .iter()
                .enumerate()
                .filter_map(|(i, n)| {
                    if i % *skip as usize == 0 {
                        Some(IndexVector {
                            index: *n as usize,
                            index_terms: vec![],
                        })
                    } else {
                        None
                    }
                })
                .collect()
            }
            Index::Random { n, seed } => {
                let mut rng: StdRng = SeedableRng::seed_from_u64(*seed as u64);
                (0..*n)
                    .map(|_| {
                        let n: usize = rng.gen_range(0, len_list);
                        IndexVector {
                            index: n,
                            index_terms: vec![],
                        }
                    })
                    .collect()
            }
            Index::IndexAndTerm { index, term } => index
                .vectorize(len_list)
                .iter_mut()
                .map(|index_vector| {
                    index_vector.index_terms.push(term.clone());
                    index_vector.to_owned()
                })
                .collect(),
        }
    }
}
