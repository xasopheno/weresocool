extern crate socool_parser;
use socool_parser::ast::Op;
use event::Event;
mod apply;
mod get_length_ratio;
mod get_operations;
mod helpers;

pub trait Apply {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self) -> f32;
}

pub trait GetOperations {
    fn get_operations(&self) -> Option<Vec<Op>>;
}

#[cfg(test)]
mod test;
