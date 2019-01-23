extern crate num_rational;
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Op {
    AsIs,
    //
    Noise,
    Sine,
    Square,
    //
    Reverse,
    FInvert,
    //
    Silence {
        m: Rational64,
    },
    TransposeM {
        m: Rational64,
    },
    TransposeA {
        a: Rational64,
    },
    PanM {
        m: Rational64,
    },
    PanA {
        a: Rational64,
    },
    Gain {
        m: Rational64,
    },
    Length {
        m: Rational64,
    },
    //
    Sequence {
        operations: Vec<Op>,
    },
    Overlay {
        operations: Vec<Op>,
    },
    Compose {
        operations: Vec<Op>,
    },
    Choice {
        operations: Vec<Op>,
    },
    ModulateBy {
        operations: Vec<Op>,
    },
    //
    WithLengthRatioOf {
        with_length_of: Box<Op>,
        main: Box<Op>,
    },
}

pub fn is_choice_op(op: Op) -> bool {
    match op {
        Op::AsIs {}
        | Op::Sine {}
        | Op::Square {}
        | Op::Noise {}
        | Op::FInvert {}
        | Op::Reverse {}
        | Op::TransposeM { .. }
        | Op::TransposeA { .. }
        | Op::PanA { .. }
        | Op::PanM { .. }
        | Op::Gain { .. }
        | Op::Length { .. }
        | Op::Silence { .. } => false,
        Op::Choice { .. } => true,

        Op::WithLengthRatioOf { .. } => false,

        Op::Sequence { operations }
        | Op::ModulateBy { operations }
        | Op::Compose { operations }
        | Op::Overlay { operations } => {
            for operation in operations {
                if is_choice_op(operation) {
                    return true;
                }
            }
            false
        }
    }
}
