use crate::generation::{RenderReturn, RenderType};

pub enum InputType<'a> {
    Filename(&'a str),
    Language(&'a str),
}

pub trait Interpretable {
    fn make(&self, target: RenderType) -> RenderReturn;
}

impl Interpretable for InputType<'_> {
    fn make(&self, _target: RenderType) -> RenderReturn {
        match &self {
            InputType::Filename(_filename) => {
                unimplemented!();
            }
            InputType::Language(_language) => {
                unimplemented!();
            }
        }
    }
}
