use error::Error;
use failure::Fail;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
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

pub fn is_import(s: String) -> bool {
    s.starts_with("import ")
}

fn handle_whitespace_and_imports(lines: Vec<String>) -> Vec<String> {
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

    imports_needed
}

#[allow(unused_variables)]
fn run() -> Result<(), Error> {
    let filename = "songs/test/template.socool";
    let mut language = "".to_string();
    let f = File::open(filename).expect("couldn't open song/test/template.socool");
    let file = BufReader::new(&f);
    file.lines().for_each(|line| {
        let l = line.expect("Could not parse line");
        language.push_str(&l);
        language.push_str("\n");
    });

    let from_filename = handle_whitespace_and_imports(filename_to_vec_string(filename));
    let from_language = handle_whitespace_and_imports(language_to_vec_string(language.as_str()));

    assert_eq!(from_language, from_filename);

    Ok(())
}
