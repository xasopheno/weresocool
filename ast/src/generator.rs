use crate::{Defs, NormalForm, Normalize, Op, Term};
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
    pub div: usize,
    pub idx: usize,
    pub axis: Axis,
    pub coefs: Vec<i64>,
    pub state: isize,
}

impl Coefs {
    fn generate(&mut self) -> Result<Op, Error> {
        self.axis.generate(self.coefs[self.idx], self.div)
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Axis {
    F,
    G,
    L,
    P,
}

fn et_to_rational(i: i64, d: usize) -> Rational64 {
    let signum = i.signum();
    if signum == 0 {
        return Rational64::from_integer(0);
    }

    let et = 2.0_f32.powf(i.abs() as f32 / d as f32);
    if signum == -1 {
        f32_to_rational(et.to_string()).recip()
    } else {
        f32_to_rational(et.to_string())
    }
}

impl Axis {
    fn generate(&self, coef: i64, div: usize) -> Result<Op, Error> {
        match self {
                Axis::F => {
                    Ok(
                        Op::TransposeM {m: et_to_rational(coef, div)}
                    )
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

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub coefs: Vec<Coefs>,
}

impl Generator {
    pub fn generate(&mut self, n: usize, defs: &Defs) -> Result<Vec<NormalForm>, Error> {
        let mut result: Vec<NormalForm> = vec![NormalForm::init()];

        for _ in 0..n - 1 {
            let mut nf: NormalForm = NormalForm::init();
            for coef in self.coefs.iter_mut() {
                coef.generate()?.apply_to_normal_form(&mut nf, defs)?;
            }
            result.push(nf)
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
