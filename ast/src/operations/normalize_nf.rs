use crate::ast::{OpOrNfTable, Term};
use crate::operations::{ArgMap, GetLengthRatio, NormalForm, Normalize, Substitute};
use num_rational::Rational64;

impl Normalize for Term {
    fn apply_to_normal_form(&self, input: &mut NormalForm, table: &OpOrNfTable) {
        match self {
            Term::Op(op) => op.apply_to_normal_form(input, table),
            Term::Nf(nf) => nf.apply_to_normal_form(input, table),
        }
    }
}

impl Substitute for Term {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        table: &OpOrNfTable,
        arg_map: &ArgMap,
    ) -> Term {
        match self {
            Term::Op(op) => op.substitute(normal_form, table, arg_map),
            Term::Nf(nf) => nf.substitute(normal_form, table, arg_map),
        }
    }
}

impl GetLengthRatio for Term {
    fn get_length_ratio(&self, table: &OpOrNfTable) -> Rational64 {
        match self {
            Term::Op(op) => op.get_length_ratio(table),
            Term::Nf(nf) => nf.get_length_ratio(table),
        }
    }
}
