use crate::generation::{parsed_to_render, RenderReturn, RenderType};
use std::path::PathBuf;
use weresocool_error::Error;
use weresocool_parser::parser::{filename_to_vec_string, language_to_vec_string, parse_file};

pub enum InputType<'a> {
    Filename(&'a str),
    Language(&'a str),
}

pub trait Interpretable {
    fn make(
        &self,
        target: RenderType,
        working_path: Option<PathBuf>,
    ) -> Result<RenderReturn, Error>;
}

impl Interpretable for InputType<'_> {
    fn make(
        &self,
        target: RenderType,
        working_path: Option<PathBuf>,
    ) -> Result<RenderReturn, Error> {
        let read_start = std::time::Instant::now();
        let (filename, vec_string) = match &self {
            InputType::Filename(filename) => (filename, filename_to_vec_string(filename)?),
            InputType::Language(language) => (&"Language", language_to_vec_string(language)),
        };
        eprintln!("[Interpretable] Read file: {:?}", read_start.elapsed());

        let parse_start = std::time::Instant::now();
        let parsed_composition = parse_file(vec_string, None, working_path)?;
        eprintln!("[Interpretable] parse_file: {:?}", parse_start.elapsed());

        let render_start = std::time::Instant::now();
        let result = parsed_to_render(filename, parsed_composition, target);
        eprintln!("[Interpretable] parsed_to_render: {:?}", render_start.elapsed());
        result
    }
}
