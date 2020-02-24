use crate::operations::helpers::{handle_id_error, join_sequence};
use crate::{Defs, GetLengthRatio, Index, Indices, ListOp, NormalForm, Normalize, Term};
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
                let indexed = get_indexed(list_nf, indices, defs);

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
                            let indexed = get_indexed(list_nf, indices, defs);

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

impl ListOp {
    pub fn to_list_nf(&self, input: &mut NormalForm, defs: &Defs) -> Vec<NormalForm> {
        match self {
            ListOp::List(operations) => {
                let mut result: Vec<NormalForm> = vec![];
                for op in operations {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs);
                    result.push(input_clone);
                }

                result
            }
            ListOp::IndexedList { terms, indices } => {
                let list_nf = normalize_list_terms(input, &terms, defs);
                let indexed = get_indexed(list_nf, indices, defs);
                indexed
            }

            ListOp::IndexedNamedList { name, indices } => {
                let lop = handle_id_error(name.to_string(), defs);
                match lop {
                    Term::Lop(list_op) => match list_op {
                        ListOp::List(terms) => {
                            let list_nf = normalize_list_terms(input, &terms, defs);
                            let indexed = get_indexed(list_nf, indices, defs);
                            indexed
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
        let vec_nf = self.to_list_nf(input, defs);
        *input = join_list_nf(vec_nf);
    }
}

fn join_list_nf(indexed: Vec<NormalForm>) -> NormalForm {
    let mut result = NormalForm::init_empty();
    for nf in indexed {
        result = join_sequence(result, nf);
    }

    result
}

fn get_indexed(list_nf: Vec<NormalForm>, indices: &Indices, defs: &Defs) -> Vec<NormalForm> {
    let mut indexed = vec![];
    match indices {
        Indices::IndexList(index_list) => {
            for index in index_list.indices.iter() {
                match index {
                    Index::Random(n, seed) => {
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
                    Index::Index(int) => indexed.push(list_nf[*int as usize].clone()),
                    Index::IndexAndTerm { index, term } => {
                        let mut nf = list_nf[*index as usize].clone();
                        term.apply_to_normal_form(&mut nf, defs);
                        indexed.push(nf);
                    }
                }
            }
        }
    }

    indexed
}

fn normalize_list_terms(nf: &NormalForm, terms: &[Term], defs: &Defs) -> Vec<NormalForm> {
    let mut list_nf = vec![];
    for term in terms {
        let mut nf = nf.clone();
        term.apply_to_normal_form(&mut nf, defs);
        list_nf.push(nf)
    }

    list_nf
}
