mod apply_to_normal_form;
mod generate;
mod get_length_ratio;
mod substitute;
use num_rational::Rational64;
use polynomials::*;
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

#[derive(Clone, PartialEq, Debug)]
pub enum Coefs {
    Const(Vec<i64>),
    Poly(Polynomial<Rational64>),
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Coefs {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Coefs::Const(c) => c.hash(state),
            Coefs::Poly(p) => p.iter().copied().collect::<Vec<Rational64>>().hash(state),
        }
    }
}

impl Into<Coefs> for Vec<Vec<i64>> {
    fn into(self) -> Coefs {
        Coefs::Const(self.into_iter().flatten().collect())
    }
}

impl Into<Coefs> for Vec<Rational64> {
    fn into(self) -> Coefs {
        let mut poly = Polynomial::new();
        for p in self {
            poly.push(p)
        }
        Coefs::Poly(poly)
    }
}

impl Coefs {
    pub fn len(&self) -> usize {
        match self {
            Coefs::Const(coefs) => coefs.len(),
            Coefs::Poly { .. } => unimplemented!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Coefs::Const(coefs) => coefs.is_empty(),
            Coefs::Poly(_coefs) => unimplemented!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
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
    println!("Using non-generator as generator.");
    Error::with_msg("Using non-generator as generator.")
}
