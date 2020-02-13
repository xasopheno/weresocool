lalrpop_mod!(pub socool);
use crate::error_handling::handle_parse_error;
use crate::imports::{get_filepath_and_import_name, is_import};
use colored::*;
use num_rational::Rational64;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use weresocool_ast::{
    ast::{Term, TermTable},
    operations::{NormalForm, Normalize},
};
use weresocool_error::{Error, ParseError};

#[derive(Clone, PartialEq, Debug)]
pub struct Init {
    pub f: Rational64,
    pub l: Rational64,
    pub g: Rational64,
    pub p: Rational64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParsedComposition {
    pub init: Init,
    pub table: TermTable,
}

fn process_op_table(ot: TermTable) -> TermTable {
    let mut result = TermTable::new();

    for (name, term) in ot.iter() {
        match term {
            Term::Nf(nf) => {
                result.insert(name.to_string(), Term::Nf(nf.to_owned()));
            }
            Term::Op(op) => {
                let mut nf = NormalForm::init();
                op.apply_to_normal_form(&mut nf, &ot);

                result.insert(name.to_string(), Term::Nf(nf));
            }
            Term::FunDef(fun) => {
                result.insert(name.to_string(), Term::FunDef(fun.to_owned()));
            }
            _ => unimplemented!(),
        };
    }

    result
}
pub fn read_file(filename: &str) -> File {
    let f = File::open(filename);
    match f {
        Ok(f) => return f,
        _ => {
            println!(
                "{} {}\n",
                "\n        File not found:".red().bold(),
                filename.red().bold()
            );

            panic!("File not found");
        }
    };
}

pub fn filename_to_vec_string(filename: &str) -> Vec<String> {
    let file = read_file(filename);
    let reader = BufReader::new(&file);
    reader
        .lines()
        .map(|line| line.expect("Could not parse line"))
        .collect()
}

pub fn language_to_vec_string(language: &str) -> Vec<String> {
    language.split('\n').map(|l| l.to_string()).collect()
}

pub fn parse_file(
    vec_string: Vec<String>,
    parse_table: Option<TermTable>,
) -> Result<ParsedComposition, ParseError> {
    let mut table = if let Some(table) = parse_table {
        table
    } else {
        TermTable::new()
    };

    let (imports_needed, composition) =
        handle_whitespace_and_imports(vec_string).expect("Whitespace and imports parsing error");

    for import in imports_needed {
        let (filepath, import_name) = get_filepath_and_import_name(import);
        let vec_string = filename_to_vec_string(&filepath.to_string());
        let parsed_composition = parse_file(vec_string, Some(table.clone()))?;

        for (key, val) in parsed_composition.table {
            let mut name = import_name.clone();
            name.push('.');
            name.push_str(&key);
            table.insert(name, val);
        }
    }

    let init = socool::SoCoolParser::new().parse(&mut table, &composition);

    match init {
        Ok(init) => {
            let table = process_op_table(table);
            Ok(ParsedComposition { init, table })
        }
        Err(error) => {
            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            let (line, column) = handle_parse_error(location, &composition);
            Err(ParseError {
                message: "Unexpected Token".to_string(),
                line,
                column,
            })
            //panic!("Unexpected Token")
        }
    }
}

fn handle_whitespace_and_imports(lines: Vec<String>) -> Result<(Vec<String>, String), Error> {
    let mut composition = String::new();
    let mut imports_needed: Vec<String> = vec![];
    for line in lines {
        let l = line;
        let copy_l = l.trim_start();
        if copy_l.starts_with("--") {
            composition.push_str("\n");
        } else if is_import(copy_l.to_string()) {
            imports_needed.push(copy_l.to_owned());
            composition.push_str("\n");
        } else {
            composition.push_str("\n");
            composition.push_str(&l);
        }
    }

    Ok((imports_needed, composition))
}

mod tests {
    #[test]
    fn filename_and_language_to_vec_string() {
        use super::*;
        let filename = "./working.socool";
        let mut language = "".to_string();
        let f = File::open(filename).expect("couldn't open ./working.socool");
        let file = BufReader::new(&f);
        file.lines().for_each(|line| {
            let l = line.expect("Could not parse line");
            language.push_str(&l);
            language.push_str("\n");
        });

        let from_filename = filename_to_vec_string(filename);
        let from_language = language_to_vec_string(language.as_str());

        for (a, b) in from_filename.iter().zip(&from_language) {
            assert_eq!(a, b);
        }
    }
}
