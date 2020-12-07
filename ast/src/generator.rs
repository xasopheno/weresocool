use crate::{Defs, NormalForm, Normalize, Term};
use num_rational::Rational64;
use weresocool_error::Error;
use weresocool_shared::helpers::r_to_f64;

use std::str::FromStr;

pub fn f32_to_rational(float_string: String) -> Rational64 {
    let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
    let den = i64::pow(10, decimal.len() as u32);
    let num = i64::from_str(&float_string.replace('.', "")).unwrap();

    Rational64::new(num, den)
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub idx: usize,
    pub axis: Axis,
    pub coefs: Vec<Rational64>,
}

impl Coefs {
    fn apply(&mut self, applicative: &mut NormalForm) -> Result<(), Error> {
        self.axis.apply(self.coefs[self.idx], applicative)?;
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
    fn apply(&self, coef: Rational64, applicative: &mut NormalForm) -> Result<(), Error> {
        for voice in applicative.operations.iter_mut() {
            for op in voice.iter_mut() {
                match self {
                    Axis::Fm => {
                        let f_64 = r_to_f64(op.fm);
                        let applied = f_64.powf(r_to_f64(coef)) as f32;
                        let applied_str = applied.to_string();
                        op.fm = f32_to_rational(applied_str);
                    }
                    _ => unimplemented!()
                    // Axis::Fa => op.fa *= coef,
                    // Axis::Lm => op.l *= coef,
                    // Axis::Gm => op.g *= coef,
                    // Axis::Pm => op.pm *= coef,
                    // Axis::Pa => op.pa *= coef,
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub idx: usize,
    pub state: NormalForm,
    pub terms: Vec<Term>,
    pub coefs: Vec<Coefs>,
}

impl Generator {
    pub fn generate(&mut self, n: usize, defs: &Defs) -> Result<Vec<NormalForm>, Error> {
        let mut state = self.state.clone();
        let mut result: Vec<NormalForm> = vec![self.state.clone()];
        let vec_nf = self
            .terms
            .iter()
            .map(|term| {
                let mut nf = NormalForm::init();
                term.apply_to_normal_form(&mut nf, defs)?;
                Ok(nf)
            })
            .collect::<Result<Vec<NormalForm>, Error>>()?;

        for i in 0..n {
            let mut applicative = vec_nf[self.idx].clone();
            for coef in self.coefs.iter_mut() {
                coef.apply(&mut applicative)?;
            }
            applicative.apply_to_normal_form(&mut state, defs)?;
            result.push(state.clone());

            self.idx += 1;
            self.idx %= self.terms.len();
        }

        Ok(result)
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
