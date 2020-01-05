use crate::generation::{RenderType, RenderReturn};

pub enum InputType {
    Filename(&str),
    Language(&str),
}

pub trait Interpretable {
    pub fn make(input: InputType, target: RenderType) -> RenderReturn;
}
