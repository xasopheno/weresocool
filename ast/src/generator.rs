use crate::operations::helpers::handle_id_error;
use crate::{Defs, NormalForm, Normalize, Op, Term};
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
    fn generate(&mut self) -> Op {
        let result = self.axis.generate(self.state, self.div);
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
    fn generate(&self, state: i64, div: usize) -> Op {
        match self {
            Axis::F => Op::TransposeM {
                m: et_to_rational(state, div),
            },
            Axis::L => Op::Length {
                m: dec_to_rational(std::cmp::max(0, state), div),
            },
            Axis::G => Op::Gain {
                m: dec_to_rational(std::cmp::max(0, state), div),
            },
            Axis::P => Op::PanA {
                a: et_to_rational(state, div),
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub coefs: Vec<Coefs>,
}

impl Generator {
    pub fn generate(
        &mut self,
        nf: &NormalForm,
        n: usize,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        let mut result: Vec<NormalForm> = vec![];

        for _ in 0..n - 1 {
            let mut nf: NormalForm = nf.clone();
            for coef in self.coefs.iter_mut() {
                coef.generate().apply_to_normal_form(&mut nf, defs)?;
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

impl GenOp {
    pub fn generate(
        self,
        n: usize,
        input: &mut NormalForm,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name, defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.generate(n, input, defs),

                    _ => {
                        println!("Using non-list as list.");
                        Err(Error::with_msg("Using non-list as list."))
                    }
                }
            }
            GenOp::Const(mut g) => g.generate(input, n.to_owned(), defs),
        }
    }
}

impl Normalize for GenOp {
    fn apply_to_normal_form(
        &self,
        _normal_form: &mut NormalForm,
        _defs: &Defs,
    ) -> Result<(), Error> {
        unimplemented!()
    }
}
