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

    let from_filename = filename_to_vec_string(filename);
    let from_language = language_to_vec_string(language.as_str());

    for (a, b) in from_filename.iter().zip(&from_language) {
        assert_eq!(a, b);
    }

    Ok(())
}
