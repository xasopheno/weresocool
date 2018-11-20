#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
//    Reverse,
    Silence {
        m: f32,
    },
    TransposeM {
        m: f32,
    },
    TransposeA {
        a: f32,
    },
//    PanM {
//        m: f32,
//    },
    PanA {
        a: f32,
    },
    Gain {
        m: f32,
    },
    Length {
        m: f32,
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
//    WithLengthRatioOf {
//        with_length_of: Box<Op>,
//        main: Box<Op>,
//    },
}

