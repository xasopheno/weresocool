extern crate num_rational;
use num_rational::Rational;

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
    Silence {
        m: Rational,
    },
    TransposeM {
        m: Rational,
    },
    TransposeA {
        a: Rational,
    },
    PanM {
        m: Rational,
    },
    PanA {
        a: Rational,
    },
    Gain {
        m: Rational,
    },
    Length {
        m: Rational
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
//
    Reverse,

    WithLengthRatioOf {
        with_length_of: Box<Op>,
        main: Box<Op>,
    },
}

