extern crate num_rational;
extern crate socool_parser;
use event::Event;
use num_rational::Rational;
use socool_parser::ast::Op;
mod apply;
mod get_length_ratio;
mod get_operations;
mod helpers;
mod normalize;
//
//#[derive(Debug, Clone)]
//pub struct Point {
//    fm: f32,
//    fa: f32,
//    pm: f32,
//    pa: f32,
//    g: f32,
//    l: f32
//}
//
//impl Point {
//    fn init() -> Point {
//        Point {
//            fm: 1.0,
//            fa: 0.0,
//            pm: 1.0,
//            pa: 0.0,
//            g: 1.0,
//            l: 1.0
//        }
//    }
//
//    fn to_op(&self) -> Op {
//        Op::Compose {
//            operations: vec![
//                Op::TransposeM { m: self.fm },
//                Op::TransposeA { a: self.fa },
//                Op::PanM { m: self.pm },
//                Op::PanA { a: self.pa },
//                Op::Gain { m: self.g },
//                Op::Length { m: self.l },
//            ]
//        }
//    }
//}

pub type NormalForm = Vec<Vec<Op>>;

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
mod test;
