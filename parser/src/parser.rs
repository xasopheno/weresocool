lalrpop_mod!(pub socool);
extern crate colored;
extern crate num_rational;
extern crate socool_ast;
use crate::error_handling::handle_parse_error;
use crate::imports::{get_filepath_and_import_name, is_import};
use colored::*;
use error::Error;
use num_rational::Rational64;
use socool_ast::{
    ast::{OpOrNf::*, OpOrNfTable},
    operations::{NormalForm, Normalize},
};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

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
    pub table: OpOrNfTable,
}

fn process_op_table(ot: OpOrNfTable) -> OpOrNfTable {
    let mut result = OpOrNfTable::new();

    for (name, op_or_nf) in ot.iter() {
        match op_or_nf {
            Nf(nf) => {
                result.insert(name.to_string(), Nf(nf.clone()));
            }
            Op(op) => {
                let mut nf = NormalForm::init();
                op.apply_to_normal_form(&mut nf, &ot);

                result.insert(name.to_string(), Nf(nf));
            }
        }
    }

    result
}

fn filename_to_vec_string(filename: &str) -> Vec<String> {
    let f = File::open(filename).expect("could open file");
    let file = BufReader::new(&f);
    file.lines()
        .map(|line| {
            let l = line.expect("Could not parse line");
            l
        })
        .collect()
}

fn language_to_vec_string(language: &str) -> Vec<String> {
    language.split("\n").map(|l| l.to_string()).collect()
}

pub fn parse_file(filename: &str, parse_table: Option<OpOrNfTable>) -> ParsedComposition {
    let mut table = if parse_table.is_some() {
        parse_table.unwrap()
    } else {
        OpOrNfTable::new()
    };

    let (imports_needed, composition) =
        handle_white_space_and_imports(filename).expect("Whitespace and imports parsing error");

    for import in imports_needed {
        let (filepath, import_name) = get_filepath_and_import_name(import);
        let parsed_composition = parse_file(&filepath.to_string(), Some(table.clone()));

        for (key, val) in parsed_composition.table {
            let mut name = import_name.clone();
            name.push('.');
            name.push_str(&key);
            table.insert(name, val);
        }
    }

    let init = socool::SoCoolParser::new().parse(&mut table, &composition);

    match init.clone() {
        Ok(init) => {
            let table = process_op_table(table);
            ParsedComposition { init, table }
        }
        Err(error) => {
            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            handle_parse_error(location, &composition);
            panic!("Unexpected Token")
        }
    }
}

fn handle_white_space_and_imports(filename: &str) -> Result<(Vec<String>, String), Error> {
    let f = File::open(filename);
    let mut composition = String::new();
    let mut imports_needed = vec![];
    match f {
        Ok(f) => {
            let file = BufReader::new(&f);
            for line in file.lines() {
                let l = line?;
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
        }
        _ => {
            println!(
                "{} {}\n",
                "\n        File not found:".red().bold(),
                filename.red().bold()
            );

            panic!("File not found");
        }
    };

    Ok((imports_needed, composition))
}

mod tests {
    #[test]
    fn import_test() {
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

