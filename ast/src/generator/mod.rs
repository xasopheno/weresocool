mod apply_to_normal_form;
mod generate;
mod get_length_ratio;
mod substitute;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum GenOp {
    Named(String),
    Const(Generator),
    Taken { gen: Box<GenOp>, n: usize },
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Generator {
    pub coefs: Vec<Coefs>,
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Coefs {
    pub axis: Axis,
    pub div: usize,
    pub idx: usize,
    pub coefs: Vec<i64>,
    pub state: i64,
    pub state_bak: i64,
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
