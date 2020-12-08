use crate::{Defs, NormalForm, Normalize, Op};
use num_rational::Rational64;
use std::str::FromStr;
use weresocool_error::Error;

pub fn f32_to_rational(float_string: String) -> Rational64 {
    dbg!(&float_string);
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
        let result = self.axis.generate(self.state, self.div);
        dbg!(self.idx, self.coefs[self.idx]);
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
    dbg!(i, d);

    let et = 2.0_f32.powf(i as f32 / d as f32);
    if signum == -1 {
        let result = f32_to_rational(format!("{:.16}", et));
        result.recip();
        result
    } else {
        f32_to_rational(format!("{:.16}", et))
    }
}

impl Axis {
    fn generate(&self, state: i64, div: usize) -> Result<Op, Error> {
        match self {
                Axis::F => {
                    Ok(
                        Op::TransposeM {m: et_to_rational(state, div)}
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
