use crate::operations::helpers::{handle_id_error, join_sequence};
use crate::{Defs, GetLengthRatio, Index, Indices, ListOp, NormalForm, Normalize, Term};
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct TermVector {
    term: Term,
    index_terms: Vec<Term>,
}

impl TermVector {
    fn get_length_ratio(&self, defs: &Defs) -> Rational64 {
        let index_terms_lr = self
            .index_terms
            .iter()
            .fold(Rational64::new(1, 1), |acc, index_term| {
                acc + index_term.get_length_ratio(defs)
            });
        self.term.get_length_ratio(defs) * index_terms_lr
    }
}

impl ListOp {
    fn term_vectors(&self, defs: &Defs) -> Vec<TermVector> {
        match self {
            ListOp::Const(terms) => terms
                .iter()
                .map(|term| TermVector {
                    term: term.to_owned(),
                    index_terms: vec![],
                })
                .collect(),
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs);
                match term {
                    Term::Lop(lop) => (lop.term_vectors(defs)),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed { list_op, indices } => {
                let mut result = vec![];
                let term_vectors = list_op.term_vectors(defs);
                let index_vectors = indices.get_indices_and_terms(term_vectors.len());
                //dbg!("%%%%%%%%%%%%%\n", &term_vectors, "_________\n");
                //dbg!("&&&&&&&&&&&&&\n", &index_vectors, "_________\n");

                for index_vector in index_vectors.iter() {
                    let mut new_index = term_vectors[index_vector.index].clone();
                    for index_term in index_vector.index_terms.iter() {
                        new_index.index_terms.push(index_term.clone());
                    }
                    result.push(new_index);
                }
                //dbg!("__________\n", &result, "_________\n");
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
                .term_vectors(defs)
                .iter()
                .fold(Rational64::new(1, 1), |acc, term_vector| {
                    acc + term_vector.get_length_ratio(defs)
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
                    Term::Lop(lop) => lop.to_list_nf(input, defs),
                    _ => panic!("Using non-list as list."),
                }
            }
            ListOp::ListOpIndexed {
                list_op: _,
                indices: _,
            } => {
                let mut result: Vec<NormalForm> = vec![];

                self.term_vectors(defs).iter_mut().for_each(|term_vector| {
                    let mut nf = input.clone();
                    term_vector.term.apply_to_normal_form(&mut nf, defs);
                    term_vector
                        .index_terms
                        .iter()
                        .for_each(|index_term| index_term.apply_to_normal_form(&mut nf, defs));
                    result.push(nf)
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

