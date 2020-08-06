use crate::generation::{parsed_to_render, RenderReturn, RenderType};
use weresocool_error::Error;
use weresocool_parser::parser::{filename_to_vec_string, language_to_vec_string, parse_file};

pub enum InputType<'a> {
    Filename(&'a str),
    Language(&'a str),
}

pub trait Interpretable {
    fn make(&self, target: RenderType) -> Result<RenderReturn, Error>;
}

impl Interpretable for InputType<'_> {
    fn make(&self, target: RenderType) -> Result<RenderReturn, Error> {
        let (filename, vec_string) = match &self {
            InputType::Filename(filename) => (filename, filename_to_vec_string(filename)),
            InputType::Language(language) => (&"Language", language_to_vec_string(language)),
        };
        let parsed_composition = parse_file(
            vec_string,
            None,
            Some("/Users/danny/Documents/weresocool/corey/"),
        )?;
        parsed_to_render(filename, parsed_composition, target)
    }
}
