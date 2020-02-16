use crate::ast::{Defs, ListOp, Term};
use crate::operations::{
    helpers::{handle_id_error, join_sequence},
    GetLengthRatio, NormalForm, Normalize,
};
use num_rational::Rational64;

impl GetLengthRatio for ListOp {
    fn get_length_ratio(&self, defs: &Defs) -> Rational64 {
        match self {
            ListOp::List(terms) => {
                let mut new_total = Rational64::from_integer(0);
                for term in terms {
                    new_total += term.get_length_ratio(defs);
                }
                new_total
            }
            ListOp::IndexedNamedList { name, indicies } => {
                let lop = handle_id_error(name.to_string(), defs);
                match lop {
                    Term::Lop(list_op) => match list_op {
                        ListOp::List(terms) => {
                            let mut new_total = Rational64::from_integer(0);
                            let nf = NormalForm::init();

                            let list_nf = normalize_list_terms(&nf, &terms, defs);
                            let indexed = get_indexed(list_nf, indicies);

                            for term in indexed {
                                new_total += term.get_length_ratio(defs);
                            }

                            new_total
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl Normalize for ListOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) {
        match self {
            ListOp::List(operations) => {
                let mut result = NormalForm::init_empty();
                for op in operations {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs);
                    result = join_sequence(result, input_clone);
                }

                *input = result
            }
            ListOp::IndexedList { terms, indicies } => {
                let list_nf = normalize_list_terms(input, &terms, defs);
                let indexed = get_indexed(list_nf, indicies);
                let joined = join_list_nf(indexed);
                *input = joined
                //unimplemented!();
            }

            ListOp::IndexedNamedList { name, indicies } => {
                let lop = handle_id_error(name.to_string(), defs);
                match lop {
                    Term::Lop(list_op) => match list_op {
                        ListOp::List(terms) => {
                            let list_nf = normalize_list_terms(input, &terms, defs);
                            let indexed = get_indexed(list_nf, indicies);
                            let joined = join_list_nf(indexed);
                            *input = joined
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            }
        }
    }
}

fn join_list_nf(indexed: Vec<NormalForm>) -> NormalForm {
    let mut result = NormalForm::init_empty();
    for nf in indexed {
        result = join_sequence(result, nf);
    }

    return result;
}

fn get_indexed(list_nf: Vec<NormalForm>, indicies: &Vec<i64>) -> Vec<NormalForm> {
    let mut indexed = vec![];
    for index in indicies {
        indexed.push(list_nf[*index as usize].clone())
    }

    indexed
}

fn normalize_list_terms(nf: &NormalForm, terms: &Vec<Term>, defs: &Defs) -> Vec<NormalForm> {
    let mut list_nf = vec![];
    for term in terms {
        let mut nf = nf.clone();
        term.apply_to_normal_form(&mut nf, defs);
        list_nf.push(nf)
    }

    list_nf
}
