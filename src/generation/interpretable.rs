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
            InputType::Filename(filename) => {
                //let vec_string = filename_to_vec_string(filename);
                unimplemented!();
            }
            InputType::Language(language) => {
                //let vec_string = language_to_vec_string(language);
                unimplemented!();
            }
        }
    }
}

