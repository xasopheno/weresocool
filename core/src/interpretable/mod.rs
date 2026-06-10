use crate::generation::{parsed_to_render, RenderReturn, RenderType};
use std::path::PathBuf;
use weresocool_error::Error;
use weresocool_parser::parser::{filename_to_vec_string, language_to_vec_string, parse_file};
use weresocool_shared::{timing_now, timing_print};

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
        let read_start = timing_now!();
        let (filename, vec_string) = match &self {
            InputType::Filename(filename) => (filename, filename_to_vec_string(filename)?),
            InputType::Language(language) => (&"Language", language_to_vec_string(language)),
        };
        timing_print!("[Interpretable] Read file: {:?}", read_start.elapsed());

        let parse_start = timing_now!();
        // For `Filename` we hand the actual path through so a parse
        // error renders a clickable `file:line:col` header; for
        // `Language` (an in-memory snippet) there is no file to point
        // at, so we suppress the header by passing None.
        let source_name = match &self {
            InputType::Filename(filename) => Some(filename.to_string()),
            InputType::Language(_) => None,
        };
        let parsed_composition = parse_file(vec_string, None, working_path, source_name)?;
        timing_print!("[Interpretable] parse_file: {:?}", parse_start.elapsed());

        let render_start = timing_now!();
        let result = parsed_to_render(filename, parsed_composition, target);
        timing_print!("[Interpretable] parsed_to_render: {:?}", render_start.elapsed());
        result
    }
}
