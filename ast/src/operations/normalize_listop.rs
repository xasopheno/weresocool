use crate::ast::{Defs, ListOp, Term};
use crate::operations::{helpers::join_sequence, NormalForm, Normalize};

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
