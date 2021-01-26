mod apply_to_normal_form;
mod generate;
mod get_length_ratio;
mod substitute;
use num_rational::Rational64;
use polynomials::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use std::hash::{Hash, Hasher};
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum GenOp {
    Named {
        name: String,
        seed: u64,
    },
    Const {
        gen: Generator,
        seed: u64,
    },
    Taken {
        gen: Box<GenOp>,
        n: usize,
        seed: u64,
    },
}

impl GenOp {
    pub fn init_named(name: String, seed: Option<(&str, i64)>) -> Self {
        let mut rng = rand::thread_rng();
        GenOp::Named {
            name,
            seed: match seed {
                None => rng.gen::<u64>(),
                Some(s) => s.1.abs() as u64,
            },
        }
    }
    pub fn init_const(gen: Generator, seed: Option<(&str, i64)>) -> Self {
        let mut rng = rand::thread_rng();
        GenOp::Const {
            gen,
            seed: match seed {
                None => rng.gen::<u64>(),
                Some(s) => s.1.abs() as u64,
            },
        }
    }
    pub fn init_taken(gen: GenOp, n: usize, seed: Option<(&str, i64)>) -> Self {
        let mut rng = rand::thread_rng();
        GenOp::Taken {
            gen: Box::new(gen),
            seed: match seed {
                None => rng.gen::<u64>(),
                Some(s) => s.1.abs() as u64,
            },
            n,
        }
    }

    pub fn set_seed(&mut self, new_seed: u64) {
        match self {
            GenOp::Named { seed, .. } => *seed = new_seed,
            GenOp::Const { seed, .. } => *seed = new_seed,
            GenOp::Taken { seed, .. } => *seed = new_seed,
        }
    }
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
    pub fn get_value(&self, mut rng: &mut StdRng) -> i64 {
        let result = match self {
            Self::Int(v) => *v,
            Self::RandRange(range) => rng.gen_range(range.to_owned()),
            Self::RandChoice(choices) => *choices.as_slice().choose(&mut rng).unwrap(),
        };
        result
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
        Self::Const(coefs.into_iter().flatten().map(Coef::Int).collect())
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

#[derive(Clone, Debug, PartialEq, Hash)]
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
    Error::with_msg("Using non-generator as generator.")
}
