use crate::ast::{Defs, Term};
use crate::operations::{ArgMap, GetLengthRatio, NormalForm, Normalize, Substitute};
use num_rational::Rational64;

impl Normalize for Term {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) {
        match self {
            Term::Op(op) => op.apply_to_normal_form(input, defs),
            Term::Nf(nf) => nf.apply_to_normal_form(input, defs),
            Term::FunDef(_fun) => unimplemented!(),
            Term::Lop(lop) => lop.apply_to_normal_form(input, defs),
            Term::Lnf(_lnf) => unimplemented!(),
        }
    }
}

impl Substitute for Term {
    fn substitute(&self, normal_form: &mut NormalForm, defs: &Defs, arg_map: &ArgMap) -> Term {
        match self {
            Term::Op(op) => op.substitute(normal_form, defs, arg_map),
            Term::Nf(nf) => nf.substitute(normal_form, defs, arg_map),
            Term::FunDef(_fun) => unimplemented!(),
            Term::Lop(_lop) => unimplemented!(),
            Term::Lnf(_lnf) => unimplemented!(),
        }
    }
}

impl GetLengthRatio for Term {
    fn get_length_ratio(&self, defs: &Defs) -> Rational64 {
        match self {
            Term::Op(op) => op.get_length_ratio(defs),
            Term::Nf(nf) => nf.get_length_ratio(defs),
            Term::FunDef(_fun) => unimplemented!(),
            Term::Lop(_lop) => unimplemented!(),
            Term::Lnf(_lnf) => unimplemented!(),
        }
    }
}
