mod apply_to_normal_form;
pub mod coefs;
mod generate;
mod get_length_ratio;
mod substitute;

use coefs::*;
use weresocool_error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum GenOp {
    Named {
        name: String,
        seed: u64,
    },
    Const {
        generator: Generator,
        seed: u64,
    },
    Taken {
        generator: Box<GenOp>,
        n: usize,
        seed: u64,
    },
}

impl GenOp {
    pub fn init_named(name: String, seed: Option<(&str, i64)>) -> Self {
        GenOp::Named {
            name,
            seed: match seed {
                None => rand::random(),
                Some((_name, seed)) => seed as u64,
            },
        }
    }
    pub fn init_const(generator: Generator, seed: Option<(&str, i64)>) -> Self {
        GenOp::Const {
            generator,
            seed: match seed {
                None => rand::random(),
                Some((_name, seed)) => seed as u64,
            },
        }
    }
    pub fn init_taken(generator: GenOp, n: usize, seed: Option<(&str, i64)>) -> Self {
        GenOp::Taken {
            generator: Box::new(generator),
            n,
            seed: match seed {
                None => rand::random(),
                Some((_name, seed)) => seed as u64,
            },
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
            div: div.unsigned_abs() as usize,
            idx: 0,
            coefs,
            axis,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum Axis {
    F,
    G,
    L,
    P,
}

pub fn error_non_generator() -> Error {
    Error::with_msg("Using non-generator as generator.")
}
