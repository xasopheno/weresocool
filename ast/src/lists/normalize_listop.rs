use crate::operations::helpers::{handle_id_error, join_sequence};
use crate::{
    Defs, GetLengthRatio, Index, IndexVector, Indices, ListOp, ListTerm, NormalForm, Normalize,
    Term,
};
use num_rational::Rational64;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

//impl ListOp {
//fn terms(&self, defs: &Defs) -> Vec<Term> {
//match self {
//ListOp::ListNf(vec_nf) => vec_nf.to_vec(),
//ListOp::ListTerm(list_term) => match list_term {
//ListTerm::Const(terms) => terms.to_vec(),
//ListTerm::Named(name) => {
//let term = handle_id_error(name.to_string(), defs);
//match term {
//Term::Lop(lop) => lop.terms(defs),
//_ => unimplemented!(),
//}
//}
//ListTerm::ListOpIndexed { list_op, indices } => unimplemented!(),
//},
//}
//}
//}

impl Indices {
    pub fn vectorize(&self) -> Vec<IndexVector> {
        self.0
            .iter()
            .map(|index| index.vectorize(self.0.len()))
            .collect()
    }
    pub fn get_length_ratio(&self, terms: Vec<Term>, defs: &Defs) -> Rational64 {
        let vectorized = self.vectorize();
        vectorized.iter().fold(Rational64::new(1, 1), |sum, index| {
            sum + index.get_length_ratio(&terms, defs)
        })
    }
}

impl Index {
    pub fn vectorize(&self, list_len: usize) -> IndexVector {
        match self {
            Index::Const(n) => IndexVector {
                indices: vec![*n],
                terms: vec![],
            },
            Index::Random(n, seed) => {
                let mut indices = vec![];
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
                    let r: usize = rng.gen_range(0, list_len);
                    indices.push(r as i64);
                }
                IndexVector {
                    indices,
                    terms: vec![],
                }
            }
            Index::IndexAndTerm { index, term } => {
                let mut vectorized = index.vectorize(list_len);
                vectorized.terms.push(term.clone());
                vectorized
            }
        }
    }
}

impl IndexVector {
    fn get_length_ratio(&self, terms: &Vec<Term>, defs: &Defs) -> Rational64 {
        let mut result = Rational64::new(1, 1);
        for term in self.terms.iter() {
            let term_lr = term.get_length_ratio(defs);
            for index in self.indices.iter() {
                result += term_lr * terms[*index as usize].get_length_ratio(defs);
            }
        }
        result
    }
}

impl GetLengthRatio for ListOp {
    fn get_length_ratio(&self, defs: &Defs) -> Rational64 {
        unimplemented!()
        //match self {
        //ListOp::ListNf(_) => unimplemented!(),
        //ListOp::ListTerm(list_term) => match list_term {
        //ListTerm::Const(terms) => {
        //let mut new_total = Rational64::from_integer(0);
        //for term in terms {
        //new_total += term.get_length_ratio(defs);
        //}
        //new_total
        //}
        //ListTerm::Named(name) => {
        //let term = handle_id_error(name.to_string(), defs);
        //match term {
        //Term::Lop(lop) => lop.get_length_ratio(defs),
        //_ => unimplemented!(),
        //}
        //}
        //ListTerm::ListOpIndexed { list_op, indices } => {
        //let terms = list_op.terms(defs);
        //indices.get_length_ratio(terms, defs)
        //}
        //}, //
        //ListOp::ListOpIndexed { listop, indices } => unimplemented!(),
        //ListOp::IndexedList { terms, indices } => {
        //let mut new_total = Rational64::from_integer(0);
        //let nf = NormalForm::init();

        //let list_nf = normalize_list_terms(&nf, &terms, defs);

        //for term in indexed {
        //new_total += term.get_length_ratio(defs);
        //}

        //new_total
        //}
        //ListOp::IndexedNamedList { name, indices } => {
        //let lop = handle_id_error(name.to_string(), defs);
        //match lop {
        //Term::Lop(list_op) => match list_op {
        //ListOp::List(terms) => {
        //let mut new_total = Rational64::from_integer(0);
        //let nf = NormalForm::init();

        //let list_nf = normalize_list_terms(&nf, &terms, defs);
        //let indexed = get_indexed(list_nf, indices, defs);

        //for term in indexed {
        //new_total += term.get_length_ratio(defs);
        //}

        //new_total
        //}
        //_ => unimplemented!(),
        //},
        //_ => unimplemented!(),
        //}
        //}
    }
}

impl ListOp {
    pub fn to_list_nf(&self, input: &mut NormalForm, defs: &Defs) -> Vec<NormalForm> {
        match self {
            ListOp::ListNf(vec_nf) => vec_nf.to_vec(),
            ListOp::ListTerm(list_term) => match list_term {
                ListTerm::Const(operations) => {
                    let mut result: Vec<NormalForm> = vec![];
                    for op in operations {
                        let mut input_clone = input.clone();
                        op.apply_to_normal_form(&mut input_clone, defs);
                        result.push(input_clone);
                    }

                    result
                }
                ListTerm::Named(name) => {
                    let term = handle_id_error(name.to_string(), defs);
                    match term {
                        Term::Lop(lop) => lop.to_list_nf(input, defs),
                        _ => panic!("Using non-list as list."),
                    }
                }
                ListTerm::ListOpIndexed { list_op, indices } => {
                    unimplemented!()
                    //let list_op_terms = list_op.terms(defs);
                    //let index_vectors = indices.vectorize();
                    //let mut result = vec![];
                    //for iv in index_vectors {
                    //for index in iv.indices {
                    //let mut nf = input.clone();
                    //let list_op_term = &list_op_terms[index as usize];
                    //list_op_term.apply_to_normal_form(&mut nf, defs);
                    //for term in &iv.terms {
                    //term.apply_to_normal_form(&mut nf, defs);
                    //}
                    //result.push(nf);
                    //}
                    //}

                    //result
                }
            },
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

fn get_indexed(
    list_op: &ListOp,
    input: &mut NormalForm,
    indices: &Indices,
    defs: &Defs,
) -> Vec<NormalForm> {
    let list_nf = list_op.to_list_nf(input, defs);

    let mut indexed = vec![];
    for index in indices.0.iter() {
        match index {
            //Index::RandomAndTerm { n, seed, term } => unimplemented!(),
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
            Index::Const(int) => indexed.push(list_nf[*int as usize].clone()),
            Index::IndexAndTerm { index, term } => {
                unimplemented!()
                //let mut nf = list_nf[*index as usize].clone();
                //term.apply_to_normal_form(&mut nf, defs);
                //indexed.push(nf);
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
