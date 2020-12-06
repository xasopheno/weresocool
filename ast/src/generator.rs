use crate::{Defs, ListOp, NormalForm, Normalize, PointOp, TermVector};
use num_rational::Rational64;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub idx: usize,
    pub axis: Axis,
    pub coefs: Vec<Rational64>,
}

impl Coefs {
    fn apply(&mut self, state: &mut NormalForm, term_vector: &TermVector) -> Result<(), Error> {
        dbg!(&self);
        for mut voice in state.operations.iter_mut() {
            for mut op in voice {
                self.axis
                    .apply(&mut op, self.coefs[self.idx], term_vector)?
            }
        }
        self.idx += 1;
        self.idx %= self.coefs.len();
        Ok(())
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

impl Axis {
    fn apply(
        &self,
        op: &mut PointOp,
        coef: Rational64,
        term_vector: &TermVector,
    ) -> Result<(), Error> {
        dbg!(&self, coef);
        match self {
            Axis::Fm => {
                panic!();
            }
            _ => unimplemented!(),
        }
        unimplemented!();
    }
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

        let term_vectors = gen.list.term_vectors(defs, None)?;
        dbg!(&term_vectors);

        let mut result: Vec<NormalForm> = vec![self.state.clone()];
        for i in 0..n {
            dbg!(i);
            for coef in self.coefs.iter_mut() {
                coef.apply(&mut gen.state, &term_vectors[self.idx])?;
            }
            result.push(gen.state.clone())
        }
        self.idx += 1;
        self.idx %= term_vectors.len();

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
