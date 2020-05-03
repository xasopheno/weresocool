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

fn make_index_error(index: i64, len_list: usize) -> Option<Error> {
    if index < 0 {
        println!("negative index {} not yet supported ¯\\_(ツ)_/¯", index);
        return Some(
            IndexError {
                index,
                len_list,
                message: format!("negative index {} not yet supported ¯\\_(ツ)_/¯", index),
            }
            .into_error(),
        );
    } else if index as usize >= len_list {
        println!(
            "index {} is greater than length of list {}",
            index, len_list
        );

        return Some(
            IndexError {
                index,
                len_list,
                message: format!(
                    "index {} is greater than length of list {}",
                    index, len_list
                ),
            }
            .into_error(),
        );
    }
    None
}

impl Index {
    pub fn vectorize(&self, len_list: usize) -> Result<Vec<IndexVector>, Error> {
        match self {
            Index::Const { indices } => indices
                .iter()
                .map(|i| {
                    let index_error = make_index_error(*i, len_list);
                    return match index_error {
                        Some(e) => Err(e),
                        None => Ok(IndexVector {
                            index: *i as usize,
                            index_terms: vec![],
                        }),
                    };
                })
                .collect::<Result<Vec<IndexVector>, Error>>(),

            Index::Slice { start, end, skip } => {
                let a = match start {
                    Some(start) => {
                        let index_error = make_index_error(*start, len_list);
                        if let Some(e) = index_error {
                            return Err(e);
                        } else {
                            *start as usize
                        }
                    }
                    None => 0,
                };
                let b = match end {
                    Some(end) => {
                        let index_error = make_index_error(*end, len_list);
                        if let Some(e) = index_error {
                            return Err(e);
                        } else {
                            *end as usize
                        }
                    }
                    None => 0,
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
