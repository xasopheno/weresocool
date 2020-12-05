use crate::{Defs, ListOp, NormalForm, Normalize, PointOp};
use num_rational::Rational64;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub idx: usize,
    pub axis: Axis,
    pub coefs: Vec<Rational64>,
}

impl Normalize for Coefs {
    fn apply_to_normal_form(&self, input: &mut NormalForm, _defs: &Defs) -> Result<(), Error> {
        dbg!(self);
        unimplemented!()
    }
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
    pub fn generate(&mut self, n: usize, defs: &Defs) -> Result<NormalForm, Error> {
        let mut gen = self.clone();
        let mut result: Vec<NormalForm> = vec![self.state.clone()];
        for i in 0..n {
            dbg!(i);
            for coef in self.coefs.iter_mut() {
                coef.apply_to_normal_form(&mut gen.state, defs)?;
            }
            result.push(gen.state.clone())
        }

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
