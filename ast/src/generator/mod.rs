mod apply_to_normal_form;
mod generate;
mod get_length_ratio;
mod substitute;
use num_rational::Rational64;
use polynomials::*;
use rand::{seq::SliceRandom, Rng};
use std::hash::{Hash, Hasher};
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum GenOp {
    Named(String),
    Const(Generator),
    Taken { gen: Box<GenOp>, n: usize },
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub coefs: Vec<CoefState>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Coef {
    Int(i64),
    RandRange(std::ops::RangeInclusive<i64>),
    RandChoice(Vec<i64>),
}

impl Coef {
    pub fn get_value(&self) -> i64 {
        match self {
            Self::Int(v) => *v,
            Self::RandRange(range) => {
                let mut rng = rand::thread_rng();
                rng.gen_range(range.to_owned())
            }
            Self::RandChoice(choices) => {
                let mut rng = rand::thread_rng();
                *choices.as_slice().choose(&mut rng).unwrap()
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Coefs {
    Const(Vec<Coef>),
    Poly(Polynomial<Rational64>),
    Expr {
        expr_str: String,
        parsed: Option<meval::Expr>,
    },
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Coefs {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Coefs::Const(c) => c.hash(state),
            Coefs::Poly(p) => p.iter().copied().collect::<Vec<Rational64>>().hash(state),
            _ => unimplemented!(),
        }
    }
}

impl Coefs {
    pub fn init_const(coefs: Vec<Vec<i64>>) -> Coefs {
        Self::Const(
            coefs
                .into_iter()
                .flatten()
                .map(|coef| Coef::Int(coef))
                .collect(),
        )
    }

    pub fn init_expr(expr_str: String) -> Coefs {
        Self::Expr {
            expr_str,
            parsed: None,
        }
    }

    pub fn init_polynomial(coefs: Vec<Rational64>) -> Coefs {
        let mut poly = Polynomial::new();
        for p in coefs {
            poly.push(p)
        }
        Coefs::Poly(poly)
    }

    pub fn len(&self) -> usize {
        match self {
            Coefs::Const(coefs) => coefs.len(),
            Coefs::Poly { .. } => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Coefs::Const(coefs) => coefs.is_empty(),
            Coefs::Poly(_coefs) => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CoefState {
    pub axis: Axis,
    pub div: usize,
    pub idx: usize,
    pub coefs: Coefs,
    pub state: i64,
    pub state_bak: i64,
}

impl CoefState {
    pub fn new(start: i64, div: i64, axis: Axis, coefs: Coefs) -> Self {
        Self {
            state: start,
            state_bak: start,
            div: div.abs() as usize,
            idx: 0,
            coefs,
            axis,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Axis {
    F,
    G,
    L,
    P,
}

pub fn error_non_generator() -> Error {
    println!("");
    Error::with_msg("Using non-generator as generator.")
}
