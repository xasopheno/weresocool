extern crate num_rational;
extern crate socool_parser;
use event::Event;
use num_rational::{Ratio, Rational};
use socool_parser::ast::Op;
mod apply;
mod get_length_ratio;
mod get_operations;
mod helpers;
mod normalize;

#[derive(Debug, Clone, PartialEq)]
pub struct PointOp {
    fm: Rational,
    fa: Rational,
    pm: Rational,
    pa: Rational,
    g: Rational,
    l: Rational,
}

impl PointOp {
    pub fn init() -> PointOp {
        PointOp {
            fm: Ratio::new(1, 1),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(0, 1),
            g: Ratio::new(1, 1),
            l: Ratio::new(1, 1),
        }
    }

    pub fn to_op(&self) -> Op {
        Op::Compose {
            operations: vec![
                Op::TransposeM { m: self.fm },
                Op::TransposeA { a: self.fa },
                Op::PanM { m: self.pm },
                Op::PanA { a: self.pa },
                Op::Gain { m: self.g },
                Op::Length { m: self.l },
            ],
        }
    }
    fn get_length_ratio(&self) -> Rational {
        self.l
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalForm {
    pub operations: Vec<Vec<PointOp>>,
    pub length_ratio: Rational,
}

impl NormalForm {
    pub fn init() -> NormalForm {
        NormalForm {
            operations: vec![vec![PointOp::init()]],
            length_ratio: Ratio::new(1, 1),
        }
    }

    pub fn init_empty() -> NormalForm {
        NormalForm {
            operations: vec![],
            length_ratio: Ratio::new(0, 1),
        }
    }

    pub fn get_length_ratio(&self) -> Rational {
        self.length_ratio
    }
}

pub trait Apply {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
}

pub trait Normalize {
    fn apply_to_normal_form(&self, normal_form: &mut NormalForm);
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self) -> Rational;
}

pub trait GetOperations {
    fn get_operations(&self) -> Option<Vec<Op>>;
}

#[cfg(test)]
mod apply_tests;
#[cfg(test)]
mod normalize_tests;
