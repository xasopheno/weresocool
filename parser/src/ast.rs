extern crate num_rational;
use num_rational::Rational64;

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
//
    Noise,
    Sine,
//
    FInvert,
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
        m: Rational64
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
        operations: Vec<Op>
    },
//
    Reverse,

    WithLengthRatioOf {
        with_length_of: Box<Op>,
        main: Box<Op>,
    },
}

