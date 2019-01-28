extern crate num_rational;
use crate::ast::{Op, OpOrNfTable, OscType};
use num_rational::{Ratio, Rational64};
use std::ops::{Mul, MulAssign};
mod get_length_ratio;
pub mod helpers;
pub mod normalize_nf;
mod normalize;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct NormalForm {
    pub operations: Vec<Vec<PointOp>>,
    pub length_ratio: Rational64,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PointOp {
    pub fm: Rational64,
    pub fa: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub g: Rational64,
    pub l: Rational64,
    pub osc_type: OscType,
}

pub trait Normalize {
    fn apply_to_normal_form(&self, normal_form: &mut NormalForm, table: &OpOrNfTable);
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self, table: &OpOrNfTable) -> Rational64;
}

impl GetLengthRatio for NormalForm {
    fn get_length_ratio(&self, _table: &OpOrNfTable) -> Rational64 {
        self.length_ratio
    }
}

impl Mul<NormalForm> for NormalForm {
    type Output = NormalForm;

    fn mul(self, other: NormalForm) -> NormalForm {
        let mut nf_result = vec![];
        let mut max_lr = Rational64::new(0, 1);
        for other_seq in other.operations.iter() {
            for other_point_op in other_seq.iter() {
                let mut seq_result: Vec<PointOp> = vec![];
                let mut seq_lr = Rational64::new(0, 1);
                for self_seq in self.operations.iter() {
                    for self_point_op in self_seq.iter() {
                        seq_lr += self_point_op.l * other_point_op.l;
                        seq_result.push(self_point_op * other_point_op);
                    }
                }

                nf_result.push(seq_result);
                if seq_lr > max_lr {
                    max_lr = seq_lr
                }
            }
        }

        NormalForm {
            operations: nf_result,
            length_ratio: max_lr,
        }
    }
}

impl<'a, 'b> Mul<NormalForm> for &'a mut NormalForm {
    type Output = NormalForm;

    fn mul(self, other: NormalForm) -> NormalForm {
        let mut nf_result = vec![];
        let mut max_lr = Rational64::new(0, 1);
        for other_seq in other.operations.iter() {
            for other_point_op in other_seq.iter() {
                let mut seq_result: Vec<PointOp> = vec![];
                let mut seq_lr = Rational64::new(0, 1);
                for self_seq in self.operations.iter() {
                    for self_point_op in self_seq.iter() {
                        seq_lr += self_point_op.l * other_point_op.l;
                        seq_result.push(self_point_op * other_point_op);
                    }
                }

                nf_result.push(seq_result);
                if seq_lr > max_lr {
                    max_lr = seq_lr
                }
            }
        }

        NormalForm {
            operations: nf_result,
            length_ratio: max_lr,
        }
    }
}

impl Normalize for NormalForm {
    fn apply_to_normal_form(&self, input: &mut NormalForm, _table: &OpOrNfTable) {
        let input_clone = input.clone();
        *input = input_clone * self.clone()
    }
}

impl Mul<PointOp> for PointOp {
    type Output = PointOp;

    fn mul(self, other: PointOp) -> PointOp {
        PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l * other.l,
            osc_type: other.osc_type,
        }
    }
}

impl<'a, 'b> Mul<&'b PointOp> for &'a PointOp {
    type Output = PointOp;

    fn mul(self, other: &'b PointOp) -> PointOp {
        PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l * other.l,
            osc_type: other.osc_type,
        }
    }
}

impl MulAssign for PointOp {
    fn mul_assign(&mut self, other: PointOp) {
        *self = PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l * other.l,
            osc_type: other.osc_type,
        }
    }
}

impl PointOp {
    pub fn mod_by(&mut self, other: PointOp) {
        *self = PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l,
            osc_type: other.osc_type,
        }
    }

    pub fn init() -> PointOp {
        PointOp {
            fm: Ratio::new(1, 1),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(0, 1),
            g: Ratio::new(1, 1),
            l: Ratio::new(1, 1),
            osc_type: OscType::Sine,
        }
    }
    pub fn init_silent() -> PointOp {
        PointOp {
            fm: Ratio::new(0, 1),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(0, 1),
            g: Ratio::new(0, 1),
            l: Ratio::new(1, 1),
            osc_type: OscType::Sine,
        }
    }

//    pub fn to_op(&self) -> Op {
//        let osc_op = match self.osc_type {
//            OscType::Sine => Op::Sine,
//            OscType::Square => Op::Square,
//            OscType::Noise => Op::Noise,
//        };
//        Op::Compose {
//            operations: vec![
//                osc_op,
//                Op::TransposeM { m: self.fm },
//                Op::TransposeA { a: self.fa },
//                Op::PanM { m: self.pm },
//                Op::PanA { a: self.pa },
//                Op::Gain { m: self.g },
//                Op::Length { m: self.l },
//            ],
//        }
//    }
}

impl NormalForm {
//    pub fn to_op(&self) -> Op {
//        let mut result = vec![];
//        for seq in self.operations.iter() {
//            let mut seq_result = vec![];
//            for p_op in seq.iter() {
//                seq_result.push(p_op.to_op())
//            }
//            result.push(Op::Sequence {
//                operations: seq_result,
//            })
//        }
//
//        Op::Overlay { operations: result }
//    }
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
}

#[cfg(test)]
mod normalize_tests;
