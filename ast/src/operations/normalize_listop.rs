use crate::ast::{Defs, Index, Indices, ListOp, Term};
use crate::operations::{
    helpers::{handle_id_error, join_sequence},
    GetLengthRatio, NormalForm, Normalize,
};
use num_rational::Rational64;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

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
            ListOp::IndexedList { terms, indices } => {
                let mut new_total = Rational64::from_integer(0);
                let nf = NormalForm::init();

                let list_nf = normalize_list_terms(&nf, &terms, defs);
                let indexed = get_indexed(list_nf, indices);

                for term in indexed {
                    new_total += term.get_length_ratio(defs);
                }

                new_total
            }
            ListOp::IndexedNamedList { name, indices } => {
                let lop = handle_id_error(name.to_string(), defs);
                match lop {
                    Term::Lop(list_op) => match list_op {
                        ListOp::List(terms) => {
                            let mut new_total = Rational64::from_integer(0);
                            let nf = NormalForm::init();

                            let list_nf = normalize_list_terms(&nf, &terms, defs);
                            let indexed = get_indexed(list_nf, indices);

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
            ListOp::IndexedList { terms, indices } => {
                let list_nf = normalize_list_terms(input, &terms, defs);
                let indexed = get_indexed(list_nf, indices);
                let joined = join_list_nf(indexed);
                *input = joined
            }

            ListOp::IndexedNamedList { name, indices } => {
                let lop = handle_id_error(name.to_string(), defs);
                match lop {
                    Term::Lop(list_op) => match list_op {
                        ListOp::List(terms) => {
                            let list_nf = normalize_list_terms(input, &terms, defs);
                            let indexed = get_indexed(list_nf, indices);
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

fn get_indexed(list_nf: Vec<NormalForm>, indices: &Indices) -> Vec<NormalForm> {
    let mut indexed = vec![];
    match indices {
        Indices::IndexList(index_list) => {
            for index in index_list.indices.iter() {
                match index {
                    Index::Index(int) => indexed.push(list_nf[*int as usize].clone()),
                }
            }
        }
        Indices::Random(n, seed) => {
            let mut rng: StdRng = match seed {
                Some(s) => SeedableRng::seed_from_u64(*s as u64),
                None => {
                    let mut rng = thread_rng();
                    let s = rng.gen::<u64>();
                    //println!("seed: {}", s);
                    SeedableRng::seed_from_u64(s as u64)
                }
            };
            for _ in 0..*n {
                let n: usize = rng.gen_range(0, list_nf.len());
                indexed.push(list_nf[n].clone());
            }
        }
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
