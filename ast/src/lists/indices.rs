use crate::{Index, IndexVector, Indices};
use rand::{rngs::StdRng, Rng, SeedableRng};
use weresocool_error::{Error, IndexError};

impl Indices {
    pub fn vectorize(&self, len_list: usize) -> Result<Vec<IndexVector>, Error> {
        let mut result = vec![];

        for index in self.0.iter() {
            let index_vectors = index.vectorize(len_list)?;
            result.extend(index_vectors);
        }

        Ok(result)
    }
}

impl Index {
    pub fn vectorize(&self, len_list: usize) -> Result<Vec<IndexVector>, Error> {
        match self {
            Index::Const { indices } => indices
                .iter()
                .map(|i| {
                    if *i as usize > len_list {
                        println!("index {} is greater than length of list {}", i, len_list);
                        Err(IndexError {
                            index: *i as usize,
                            len_list,
                            message: format!(
                                "index {} is greater than length of list {}",
                                i, len_list
                            ),
                        }
                        .to_error())
                    } else {
                        Ok(IndexVector {
                            index: *i as usize,
                            index_terms: vec![],
                        })
                    }
                })
                .collect::<Result<Vec<IndexVector>, Error>>(),

            Index::Slice { start, end, skip } => {
                let a = match start {
                    Some(start) => {
                        if *start as usize > len_list as usize {
                            println!(
                                "Start of slice {} is greater than length of list {}",
                                start, len_list
                            );
                            return Err(IndexError {
                                index: *start as usize,
                                len_list,
                                message: format!(
                                    "index {} is greater than length of list {}",
                                    *start, len_list
                                ),
                            }
                            .to_error());
                        }

                        *start as usize
                    }
                    None => 0,
                };
                let b = match end {
                    Some(end) => {
                        if (*end as usize) > len_list as usize {
                            println!(
                                "End of slice {} is greater than length of list {}",
                                end, len_list
                            );

                            return Err(IndexError {
                                index: *end as usize,
                                len_list,
                                message: format!(
                                    "index {} is greater than length of list {}",
                                    *end, len_list
                                ),
                            }
                            .to_error());
                        }
                        *end as usize
                    }
                    None => (len_list - 1) as usize,
                };

                Ok(if a < b {
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
                .collect())
            }
            Index::Random { n, seed } => {
                let mut rng: StdRng = SeedableRng::seed_from_u64(*seed as u64);
                Ok((0..*n)
                    .map(|_| {
                        let n: usize = rng.gen_range(0, len_list);
                        IndexVector {
                            index: n,
                            index_terms: vec![],
                        }
                    })
                    .collect())
            }
            Index::IndexAndTerm { index, term } => Ok(index
                .vectorize(len_list)?
                .iter_mut()
                .map(|index_vector| {
                    index_vector.index_terms.push(term.clone());
                    index_vector.to_owned()
                })
                .collect()),
        }
    }
}
