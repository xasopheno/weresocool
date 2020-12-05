use crate::{Defs, ListOp, NormalForm, Normalize};
use num_rational::Rational64;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub idx: usize,
    pub axis: Axis,
    pub coefs: Vec<Rational64>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Axis {
    Fm,
    Fa,
    Gm,
    Lm,
    Pm,
    Pa,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub idx: usize,
    pub state: NormalForm,
    pub list: ListOp,
    pub coefs: Vec<Coefs>,
}

impl Generator {
    pub fn generate(&mut self, n: usize) -> NormalForm {
        dbg!(self, n);
        unimplemented!()
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum GenOp {
    Const(Generator),
    Named(String),
}

impl Normalize for GenOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) -> Result<(), Error> {
        unimplemented!();
    }
}
