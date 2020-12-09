use crate::{Defs, NormalForm, Normalize, Op};
use num_rational::Rational64;
use std::str::FromStr;
use weresocool_error::Error;

pub fn f32_to_rational(float_string: String) -> Rational64 {
    let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
    let den = i64::pow(10, decimal.len() as u32);
    let num = i64::from_str(&float_string.replace('.', "")).unwrap();

    Rational64::new(num, den)
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub axis: Axis,
    pub div: usize,
    pub idx: usize,
    pub coefs: Vec<i64>,
    pub state: i64,
}

impl Coefs {
    fn generate(&mut self) -> Result<Op, Error> {
        let result = self
            .axis
            .generate(self.state, self.div, self.coefs[self.idx]);
        self.state += self.coefs[self.idx];
        self.idx += 1;
        self.idx %= self.coefs.len();
        result
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
        return Rational64::from_integer(1);
    }

    let et = 2.0_f32.powf(i as f32 / d as f32);
    if signum == -1 {
        let result = f32_to_rational(format!("{:.16}", et));
        result.recip();
        result
    } else {
        f32_to_rational(format!("{:.16}", et))
    }
}

fn dec_to_rational(i: i64, d: usize) -> Rational64 {
    Rational64::new(i, d as i64)
}

impl Axis {
    fn generate(&self, state: i64, div: usize, coef: i64) -> Result<Op, Error> {
        match self {
            Axis::F => Ok(Op::TransposeM {
                m: et_to_rational(state, div),
            }),
            Axis::L => {
                let m;
                if state < 0 {
                    m = dec_to_rational(1, div);
                } else {
                    m = dec_to_rational(coef, div);
                }
                Ok(Op::Length { m })
            }
            // Axis::G => op.g *= coef,
            // Axis::P => op.pm *= coef,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub coefs: Vec<Coefs>,
}

impl Generator {
    pub fn generate(&mut self, n: usize, defs: &Defs) -> Result<Vec<NormalForm>, Error> {
        let mut result: Vec<NormalForm> = vec![];

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
