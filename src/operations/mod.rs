use event::Event;
mod apply;
mod apply_with_order;
mod get_length_ratio;
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
    ComposeWithOrder {
        operations: Vec<Op>,
        order_fn: fn(usize) -> f32,
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

pub trait Apply {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
}

pub trait ApplyWithOrder {
    fn apply_with_order(&self, order_fn: fn(order: usize) -> f32, events: Vec<Event>)
        -> Vec<Event>;
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self) -> f32;
}

#[cfg(test)]
mod test;
