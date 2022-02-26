use crate::{FunDef, GenOp, GetLengthRatio, ListOp, NormalForm, Normalize, Op, Substitute};
use num_rational::Rational64;
use scop::Defs;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Term {
    Op(Op),
    Nf(NormalForm),
    FunDef(FunDef),
    Lop(ListOp),
    Gen(GenOp),
}

impl Normalize<Term> for Term {
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        defs: &mut Defs<Term>,
    ) -> Result<(), Error> {
        match self {
            Term::Op(op) => op.apply_to_normal_form(input, defs),
            Term::Nf(nf) => nf.apply_to_normal_form(input, defs),
            Term::FunDef(_fun) => Err(Error::with_msg("Cannot normalize FunDef.")),
            Term::Lop(lop) => lop.apply_to_normal_form(input, defs),
            Term::Gen(gen) => gen.apply_to_normal_form(input, defs),
        }
    }
}

impl Substitute<Term> for Term {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs<Term>,
    ) -> Result<Term, Error> {
        match self {
            Term::Op(op) => op.substitute(normal_form, defs),
            Term::Nf(nf) => nf.substitute(normal_form, defs),
            Term::FunDef(_fun) => Err(Error::with_msg("Cannot call substitute on FunDef.")),
            Term::Lop(lop) => lop.substitute(normal_form, defs),
            Term::Gen(gen) => gen.substitute(normal_form, defs),
        }
    }
}

impl GetLengthRatio<Term> for Term {
    fn get_length_ratio(
        &self,
        normal_form: &NormalForm,
        defs: &mut Defs<Term>,
    ) -> Result<Rational64, Error> {
        match self {
            Term::Op(op) => op.get_length_ratio(normal_form, defs),
            Term::Nf(nf) => nf.get_length_ratio(normal_form, defs),
            Term::FunDef(_fun) => Err(Error::with_msg("Cannot get length_ratio of FunDef.")),
            Term::Lop(lop) => lop.get_length_ratio(normal_form, defs),
            Term::Gen(gen) => gen.get_length_ratio(normal_form, defs),
        }
    }
}
