use event::Event;
mod apply;
mod get_length_ratio;
mod helpers;
//#[derive(Clone, PartialEq, Debug)]
//pub enum Op {
//    AsIs,
//    Reverse,
//    PanM {
//        m: f32,
//    },
//    PanA {
//        a: f32,
//    },
//    TransposeM {
//        m: f32,
//    },
//    TransposeA {
//        a: f32,
//    },
//    Silence {
//        m: f32,
//    },
//    Length {
//        m: f32,
//    },
//    Gain {
//        m: f32,
//    },
//
//    Sequence {
//        operations: Vec<Op>,
//    },
//    Compose {
//        operations: Vec<Op>,
//    },
//    Overlay {
//        operations: Vec<Op>,
//    },
//
//    WithLengthRatioOf { with_length_of: Box<Op>, main: Box<Op> },
//}
//
pub trait Apply {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self) -> f32;
}


#[cfg(test)]
mod test;
