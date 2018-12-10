extern crate num_rational;
extern crate socool_parser;
use num_rational::{Ratio, Rational64};
use instrument::oscillator::OscType;
use socool_parser::ast::Op;
mod get_length_ratio;
mod helpers;
mod normalize;

#[derive(Debug, Clone, PartialEq)]
pub struct PointOp {
    pub fm: Rational64,
    pub fa: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub g: Rational64,
    pub l: Rational64,
    pub osc_type: OscType,
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
            osc_type: OscType::Sine
        }
    }

    pub fn to_op(&self) -> Op {
        let osc_op = match self.osc_type  {
            OscType::Sine => Op::Sine,
            OscType::Noise => Op::Noise,
        };
        Op::Compose {
            operations: vec![
                osc_op,
                Op::TransposeM { m: self.fm },
                Op::TransposeA { a: self.fa },
                Op::PanM { m: self.pm },
                Op::PanA { a: self.pa },
                Op::Gain { m: self.g },
                Op::Length { m: self.l },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalForm {
    pub operations: Vec<Vec<PointOp>>,
    pub length_ratio: Rational64,
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

    pub fn get_length_ratio(&self) -> Rational64 {
        self.length_ratio
    }
}

pub trait Normalize {
    fn apply_to_normal_form(&self, normal_form: &mut NormalForm);
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self) -> Rational64;
}

#[cfg(test)]
mod normalize_tests;
