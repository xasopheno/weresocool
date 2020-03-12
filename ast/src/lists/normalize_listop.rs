use crate::operations::helpers::{handle_id_error, join_sequence};
use crate::{ArgMap, Defs, GetLengthRatio, ListOp, NormalForm, Normalize, Term, TermVector};
use num_rational::Rational64;

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
                let mut result = vec![];
                let term_vectors = list_op.term_vectors(defs, arg_map);
                let index_vectors = indices.get_indices_and_terms(term_vectors.len());

                for index_vector in index_vectors.iter() {
                    let mut new_index = term_vectors[index_vector.index].clone();
                    for index_term in index_vector.index_terms.iter() {
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
            ListOp::Const(terms) => terms.iter().fold(Rational64::from_integer(0), |acc, term| {
                acc + term.get_length_ratio(defs)
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
                self.apply_to_normal_form(&mut nf, defs);
                nf.get_length_ratio(defs)
            }
        }
    }
}

impl ListOp {
    pub fn to_list_nf(&self, input: &mut NormalForm, defs: &Defs) -> Vec<NormalForm> {
        match self {
            ListOp::Const(operations) => operations
                .iter()
                .map(|op| {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs);
                    input_clone
                })
                .collect(),
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
                    term_vector.term.apply_to_normal_form(&mut nf, defs);
                    term_vector
                        .index_terms
                        .iter()
                        .for_each(|index_term| index_term.apply_to_normal_form(&mut nf, defs));
                    nf
                })
                .collect(),
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
