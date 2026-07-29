//! Text DSL — `text NAME = { "STRING", size, at, tracking }`.
//!
//! Parse + extraction only. The letterforms and the layout live in kintaro
//! (`crate::text`), the same split every visual DSL uses.

pub mod ast;
pub mod preprocess;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub text_grammar,
    "/text_def/text_grammar.rs"
);

pub use ast::{TextDef, TextField};
pub use preprocess::{extract_texts, TextPreprocessError, TextPreprocessed};
