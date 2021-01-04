use crate::operations::helpers::handle_id_error;
use crate::{
    lists::normalize_listop::join_list_nf, ArgMap, Defs, NormalForm, Normalize, Op, Substitute,
    Term,
};
use num_integer::lcm;
use num_rational::Rational64;
// use std::str::FromStr;
use weresocool_error::Error;
use weresocool_shared::helpers::f32_string_to_rational;

// pub fn f32_to_rational(float_string: String) -> Rational64 {
// let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
// let den = i64::pow(10, decimal.len() as u32);
// let num = i64::from_str(&float_string.replace('.', "")).unwrap();

// Rational64::new(num, den)
// }

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
        let result = f32_string_to_rational(format!("{:.4}", et));
        result.recip();
        result
    } else {
        f32_string_to_rational(format!("{:.4}", et))
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
    fn lcm_length(&self) -> usize {
        let lengths: Vec<usize> = self.coefs.iter().map(|coef| coef.coefs.len()).collect();
        1 + lengths
            .iter()
            .fold(1usize, |current, val| lcm(current, *val))
    }
    pub fn generate(
        &mut self,
        nf: &NormalForm,
        n: usize,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        let mut result: Vec<NormalForm> = vec![];

        for _ in 0..n {
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
    Named(String),
    Const(Generator),
    Taken { gen: Box<GenOp>, n: usize },
}

impl Normalize for GenOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) -> Result<(), Error> {
        match self {
            GenOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, None)?;
                match term {
                    Term::Gen(gen) => gen.apply_to_normal_form(input, defs),
                    _ => Err(Error::with_msg("List.term_vectors() called on non-list")),
                }
            }
            GenOp::Const(gen) => {
                let lcm_length = gen.lcm_length();
                *input = join_list_nf(gen.to_owned().generate(input, lcm_length, defs)?);
                Ok(())
            }
            GenOp::Taken { n, gen } => {
                *input = join_list_nf(gen.to_owned().generate_from_genop(input, Some(*n), defs)?);
                Ok(())
            }
        }
    }
}

impl GenOp {
    pub fn length(&self, defs: &Defs) -> Result<usize, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name.to_string(), defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.length(defs),

                    _ => {
                        println!("Using non-generator as generator.");
                        Err(Error::with_msg("Using non-list as list."))
                    }
                }
            }
            GenOp::Const(generator) => Ok(generator.lcm_length()),
            GenOp::Taken { n, .. } => Ok(*n),
        }
    }
    pub fn generate_from_genop(
        self,
        input: &mut NormalForm,
        n: Option<usize>,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name, defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.generate_from_genop(input, n, defs),

                    _ => {
                        println!("Using non-generator as generator.");
                        Err(Error::with_msg("Using non-list as list."))
                    }
                }
            }
            GenOp::Const(mut gen) => {
                let length = if n.is_some() {
                    n.unwrap()
                } else {
                    gen.lcm_length()
                };
                gen.generate(input, length, defs)
            }
            GenOp::Taken { gen, n } => gen.to_owned().generate_from_genop(input, Some(n), defs),
        }
    }
}

impl Substitute for GenOp {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &Defs,
        arg_map: &ArgMap,
    ) -> Result<Term, Error> {
        match self {
            GenOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, Some(arg_map))?;
                match term {
                    Term::Gen(_) => Ok(term),

                    _ => {
                        println!("Using non-generator as generator.");
                        Err(Error::with_msg("Using non-list as list."))
                    }
                }
            }
            GenOp::Const(_) => Ok(Term::Gen(self.to_owned())),
            GenOp::Taken { n, gen } => {
                let term = gen.substitute(normal_form, defs, arg_map)?;
                match term {
                    Term::Gen(gen) => Ok(Term::Gen(GenOp::Taken {
                        n: *n,
                        gen: Box::new(gen),
                    })),

                    _ => {
                        println!("Using non-generator as generator.");
                        Err(Error::with_msg("Using non-list as list."))
                    }
                }
            }
        }
    }
}
