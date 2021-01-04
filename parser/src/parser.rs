lalrpop_mod!(pub socool);
use crate::error_handling::handle_parse_error;
use crate::imports::{get_filepath_and_import_name, is_import};
use colored::*;
use num_rational::Rational64;
use path_clean::PathClean;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use weresocool_ast::{Defs, NormalForm, Normalize, Term};
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
    pub defs: Defs,
}

fn process_op_table(defs: Defs) -> Result<Defs, Error> {
    let mut result: Defs = Default::default();

    for (name, term) in defs.terms.iter() {
        match term {
            Term::Nf(nf) => {
                result
                    .terms
                    .insert(name.to_string(), Term::Nf(nf.to_owned()));
            }
            Term::Op(op) => {
                let mut nf = NormalForm::init();
                op.apply_to_normal_form(&mut nf, &defs)?;

                result.terms.insert(name.to_string(), Term::Nf(nf));
            }
            Term::FunDef(fun) => {
                result
                    .terms
                    .insert(name.to_string(), Term::FunDef(fun.to_owned()));
            }
            Term::Lop(lop) => {
                let mut nf = NormalForm::init();
                lop.apply_to_normal_form(&mut nf, &defs)?;
                result.terms.insert(name.to_string(), Term::Nf(nf));
            }
            Term::Gen(gen) => {
                let mut nf = NormalForm::init();
                gen.apply_to_normal_form(&mut nf, &defs)?;

                result.terms.insert(name.to_string(), Term::Nf(nf));
            }
        };
    }

    Ok(result)
}
pub fn read_file(filename: &str) -> Result<File, Error> {
    let f = File::open(filename);
    match f {
        Ok(f) => return Ok(f),
        _ => {
            println!(
                "{} {}\n",
                "\n        File not found:".red().bold(),
                filename.red().bold()
            );

            return Err(Error::with_msg(format!("File not found: {}", filename)));
        }
    };
}

pub fn filename_to_vec_string(filename: &str) -> Result<Vec<String>, Error> {
    let file = read_file(filename)?;
    let reader = BufReader::new(&file);
    Ok(reader
        .lines()
        .map(|line| line)
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn language_to_vec_string(language: &str) -> Vec<String> {
    language.split('\n').map(|l| l.to_string()).collect()
}

pub fn parse_file(
    vec_string: Vec<String>,
    prev_defs: Option<Defs>,
    working_path: Option<String>,
) -> Result<ParsedComposition, Error> {
    let mut defs: Defs = if let Some(defs) = prev_defs {
        defs
    } else {
        Default::default()
    };

    let (imports_needed, composition) = handle_whitespace_and_imports(vec_string)?;
    for import in imports_needed {
        let (mut filepath, import_name) = get_filepath_and_import_name(import);
        if let Some(wd) = working_path.clone() {
            let mut pb = std::path::PathBuf::new();
            pb.push(wd);
            pb.push(filepath);
            filepath = pb.clean().display().to_string();
        }
        dbg!(&filepath);
        let vec_string = filename_to_vec_string(&filepath.to_string())?;
        let parsed_composition = parse_file(vec_string, Some(defs.clone()), working_path.clone())?;

        for (key, val) in parsed_composition.defs.terms {
            let mut name = import_name.clone();
            name.push('.');
            name.push_str(&key);
            defs.terms.insert(name, val);
        }

        for (key, val) in parsed_composition.defs.lists {
            let mut name = import_name.clone();
            name.push('.');
            name.push_str(&key);
            defs.lists.insert(name, val);
        }
    }

    let init = socool::SoCoolParser::new().parse(&mut defs, &composition);

    match init {
        Ok(init) => {
            let defs = process_op_table(defs)?;
            Ok(ParsedComposition { init, defs })
        }
        Err(error) => {
            let location = Arc::new(Mutex::new(Vec::new()));
            error.map_location(|l| location.lock().unwrap().push(l));
            let (line, column) = handle_parse_error(location, &composition);

            Err(ParseError {
                message: "Unexpected Token".to_string(),
                line,
                column,
            }
            .into_error())
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

        let from_filename = filename_to_vec_string(filename).unwrap();
        let from_language = language_to_vec_string(language.as_str());

        for (a, b) in from_filename.iter().zip(&from_language) {
            assert_eq!(a, b);
        }
    }
}
