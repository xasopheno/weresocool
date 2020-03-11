use crate::{Index, IndexVector, Indices};
use rand::{rngs::StdRng, Rng, SeedableRng};

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

            Index::Slice { start, end } => {
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
                    None => len_list as usize,
                };

                if a == b {
                    panic! {"start {} and end {} of slice are the same value", a, b};
                };

                let mut result = vec![];
                if a < b {
                    for n in a..b + 1 {
                        result.push(IndexVector {
                            index: n as usize,
                            index_terms: vec![],
                        });
                    }
                } else {
                    for n in (b..a + 1).rev() {
                        result.push(IndexVector {
                            index: n as usize,
                            index_terms: vec![],
                        });
                    }
                };

                result
            }
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
