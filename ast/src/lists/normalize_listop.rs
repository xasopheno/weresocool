use crate::operations::helpers::{handle_id_error, join_sequence};
use crate::{Defs, GetLengthRatio, Index, Indices, ListOp, NormalForm, Normalize, Term};
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct IndexVector {
    terms: Vec<Term>,
    index_terms: Vec<Term>,
}

impl IndexVector {
    fn get_length_ratio(&self, defs: &Defs) -> Rational64 {
        let index_terms_lr = self
            .index_terms
            .iter()
            .fold(Rational64::new(1, 1), |acc, index_term| {
                acc + index_term.get_length_ratio(defs)
            });
        self.terms.iter().fold(Rational64::new(1, 1), |acc, term| {
            acc + (term.get_length_ratio(defs) * index_terms_lr)
        })
    }
    pub fn to_list_nf(&mut self, input: &mut NormalForm, defs: &Defs) -> Vec<NormalForm> {
        self.terms
            .iter()
            .map(|term| {
                let mut nf = input.clone();
                term.apply_to_normal_form(input, defs);
                self.index_terms
                    .iter()
                    .for_each(|index_term| index_term.apply_to_normal_form(&mut nf, defs));
                nf
            })
            .collect()
    }
}

impl ListOp {
    fn terms(&self, defs: &Defs) -> Vec<IndexVector> {
        match self {
            ListOp::Const(terms) => vec![IndexVector {
                terms: terms.to_vec(),
                index_terms: vec![],
            }],
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs);
                match term {
                    Term::Lop(lop) => (lop.terms(defs)),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed { list_op, indices } => {
                let mut result = vec![];
                let index_vectors = list_op.terms(defs);
                let (indices, index_terms) = indices.get_indices_and_terms(index_vectors.len());
                for index in indices {
                    let mut new_index = index_vectors[index].clone();
                    for index_term in index_terms.iter() {
                        new_index.index_terms.push(index_term.clone());
                    }
                    result.push(new_index);
                }
                result
            }
        }
    }
}

impl GetLengthRatio for ListOp {
    fn get_length_ratio(&self, defs: &Defs) -> Rational64 {
        match self {
            ListOp::Const(terms) => {
                let mut new_total = Rational64::from_integer(0);
                for term in terms {
                    new_total += term.get_length_ratio(defs);
                }
                new_total
            }
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs);
                match term {
                    Term::Lop(lop) => lop.get_length_ratio(defs),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed {
                list_op: _,
                indices: _,
            } => self
                .terms(defs)
                .iter()
                .fold(Rational64::new(1, 1), |acc, index_vector| {
                    acc + index_vector.get_length_ratio(defs)
                }),
        }
    }
}

impl ListOp {
    pub fn to_list_nf(&self, input: &mut NormalForm, defs: &Defs) -> Vec<NormalForm> {
        match self {
            ListOp::Const(operations) => {
                let mut result: Vec<NormalForm> = vec![];
                for op in operations {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs);
                    result.push(input_clone);
                }
                result
            }
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs);
                match term {
                    Term::Lop(mut lop) => lop.to_list_nf(input, defs),
                    _ => panic!("Using non-list as list."),
                }
            }
            ListOp::ListOpIndexed {
                list_op: _,
                indices: _,
            } => {
                let mut result: Vec<NormalForm> = vec![];
                self.terms(defs).iter_mut().for_each(|index_term| {
                    let list_nf = index_term.to_list_nf(input, defs);
                    for nf in list_nf.iter() {
                        result.push(nf.clone())
                    }
                });
                result
            }
        }
    }
}

impl Normalize for ListOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) {
        let list_nf = self.to_list_nf(input, defs);
        *input = join_list_nf(list_nf);
    }
}

fn join_list_nf(indexed: Vec<NormalForm>) -> NormalForm {
    let mut result = NormalForm::init_empty();
    for nf in indexed {
        result = join_sequence(result, nf);
    }

    result
}

