mod apply_to_normal_form;
mod generate;
mod get_length_ratio;
mod substitute;
use num_rational::Rational64;
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
pub enum Coefs {
    Const(Vec<i64>),
    // Poly(Vec<Rational64>),
}

impl Coefs {
    pub fn len(&self) -> usize {
        match self {
            Coefs::Const(coefs) => coefs.len(),
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
    pub fn new(start: i64, div: usize, axis: Axis, coefs: Coefs) -> Self {
        Self {
            state: start,
            state_bak: start,
            div,
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
