use crate::operations::helpers::{handle_id_error, join_sequence};
use crate::{ArgMap, Defs, GetLengthRatio, ListOp, NormalForm, Normalize, Term, TermVector};
use num_rational::Rational64;
use weresocool_error::Error;

impl ListOp {
    pub fn term_vectors(&self, defs: &Defs, arg_map: Option<&ArgMap>) -> Vec<TermVector> {
        match self {
            ListOp::Const(terms) => terms
                .iter()
                .map(|term| TermVector {
                    term: term.to_owned(),
                    index_terms: vec![],
                })
                .collect(),
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, arg_map);
                match term {
                    Term::Lop(lop) => (lop.term_vectors(defs, arg_map)),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed { list_op, indices } => {
                let term_vectors = list_op.term_vectors(defs, arg_map);
                let index_vectors = indices.vectorize(term_vectors.len());

                index_vectors
                    .iter()
                    .map(|index_vector| {
                        let mut new_index = term_vectors[index_vector.index].clone();
                        index_vector.index_terms.iter().for_each(|index_term| {
                            new_index.index_terms.push(index_term.clone());
                        });
                        new_index
                    })
                    .collect()
            }
            ListOp::Concat(lists) => {
                let mut result = vec![];
                for list in lists {
                    result.extend(list.term_vectors(defs, arg_map))
                }

                result
            }
        }
    }
}

impl GetLengthRatio for ListOp {
    fn get_length_ratio(&self, defs: &Defs) -> Result<Rational64, Error> {
        match self {
            ListOp::Const(terms) => terms
                .iter()
                .try_fold(Rational64::from_integer(0), |acc, term| {
                    Ok(acc + term.get_length_ratio(defs)?)
                }),
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, None);
                match term {
                    Term::Lop(lop) => lop.get_length_ratio(defs),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;
                nf.get_length_ratio(defs)
            }
            ListOp::Concat(listops) => listops
                .iter()
                .try_fold(Rational64::from_integer(0), |acc, term| {
                    Ok(acc + term.get_length_ratio(defs)?)
                }),
        }
    }
}

impl ListOp {
    pub fn to_list_nf(
        &self,
        input: &mut NormalForm,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        match self {
            ListOp::Const(operations) => operations
                .iter()
                .map(|op| {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs)?;
                    Ok(input_clone)
                })
                .collect::<Result<Vec<NormalForm>, Error>>(),

            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, None);
                match term {
                    Term::Lop(lop) => lop.to_list_nf(input, defs),
                    _ => panic!("Using non-list as list."),
                }
            }
            ListOp::ListOpIndexed { .. } => self
                .term_vectors(defs, None)
                .iter_mut()
                .map(|term_vector| {
                    let mut nf = input.clone();
                    term_vector.term.apply_to_normal_form(&mut nf, defs)?;
                    term_vector
                        .index_terms
                        .iter()
                        .map(|index_term| {
                            index_term.apply_to_normal_form(&mut nf, defs)?;
                            Ok(())
                        })
                        .collect::<Result<Vec<_>, Error>>()?;
                    Ok(nf)
                })
                .collect::<Result<Vec<NormalForm>, Error>>(),
            ListOp::Concat(listops) => listops
                .iter()
                .map(|list| {
                    let mut nf = input.clone();
                    list.apply_to_normal_form(&mut nf, defs)?;
                    Ok(nf)
                })
                .collect(),
        }
    }
}

impl Normalize for ListOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) -> Result<(), Error> {
        *input = join_list_nf(self.to_list_nf(input, defs)?);
        Ok(())
    }
}

fn join_list_nf(indexed: Vec<NormalForm>) -> NormalForm {
    indexed.iter().fold(NormalForm::init_empty(), |acc, nf| {
        join_sequence(acc, nf.to_owned())
    })
}
