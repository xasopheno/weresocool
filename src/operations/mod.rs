use event::Event;
use std::cmp;
mod operate;
mod helpers;

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
    Reverse,
    PanM {
        m: f32,
    },
    PanA {
        a: f32,
    },
    TransposeM {
        m: f32,
    },
    TransposeA {
        a: f32,
    },
    Silence {
        m: f32,
    },
    Length {
        m: f32,
    },
    Gain {
        m: f32,
    },
    //    Capture { n: usize },
    Repeat {
        n: usize,
        operations: Vec<Op>,
    },
    Sequence {
        operations: Vec<Op>,
    },
    Compose {
        operations: Vec<Op>,
    },
    Fit {
        with_length_of: Box<Op>,
        main: Box<Op>,
        n: usize,
    },
    Overlay {
        operations: Vec<Op>,
    },
}

pub trait Operate {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
    fn get_length_ratio(&self) -> f32;

}

#[cfg(test)]
mod test;
