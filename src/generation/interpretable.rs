use crate::generation::{RenderReturn, RenderType};
use socool_parser::parser::{filename_to_vec_string, language_to_vec_string, parse_file};

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
                let vec_string = filename_to_vec_string(filename);
                let parsed_composition = parse_file(vec_string, None);
                unimplemented!();
            }
            InputType::Language(language) => {
                let vec_string = language_to_vec_string(language);
                let parsed_composition = parse_file(vec_string, None);
                unimplemented!();
            }
        }
    }
}

