use crate::{FunDef, GenOp, GetLengthRatio, ListOp, NormalForm, Normalize, Op, Substitute, Defs};
use num_rational::Rational64;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Term {
    Op(Op),
    Nf(NormalForm),
    FunDef(FunDef),
    Lop(ListOp),
    Gen(GenOp),
}

impl Normalize for Term {
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<(), Error> {
        match self {
            Term::Nf(nf) => nf.apply_to_normal_form(input, defs),
            Term::Op(op) => op.apply_to_normal_form(input, defs),
            Term::FunDef(_) => Err(Error::with_msg("FunDef should not be normalized")),
            Term::Lop(lop) => lop.apply_to_normal_form(input, defs),
            Term::Gen(generator) => generator.apply_to_normal_form(input, defs),
        }
    }
}

impl Substitute for Term {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<Term, Error> {
        match self {
            Term::Op(op) => op.substitute(normal_form, defs),
            Term::Nf(_) => Ok(self.to_owned()),
            Term::FunDef(_) => Ok(self.to_owned()),
            Term::Lop(lop) => lop.substitute(normal_form, defs),
            Term::Gen(generator) => generator.substitute(normal_form, defs),
        }
    }
}

impl GetLengthRatio for Term {
    fn get_length_ratio(
        &self,
        normal_form: &NormalForm,
        defs: &mut Defs,
    ) -> Result<Rational64, Error> {
        match self {
            Term::Op(op) => op.get_length_ratio(normal_form, defs),
            Term::Nf(nf) => nf.get_length_ratio(normal_form, defs),
            Term::FunDef(_) => Err(Error::with_msg("Cannot get length ratio of FunDef.")),
            Term::Lop(lop) => lop.get_length_ratio(normal_form, defs),
            Term::Gen(generator) => generator.get_length_ratio(normal_form, defs),
        }
    }
}
