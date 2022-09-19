use crate::play::{play_file, Play};
use crate::Error;
use clap::ArgMatches;
use indoc::indoc;
use std::fs;
use std::path::PathBuf;

pub fn new(play_args: Option<&ArgMatches>, cwd: PathBuf) -> Result<(), Error> {
    let filename = play_args
        .ok_or_else(|| Error::Message("filename required".to_string()))?
        .values_of("file")
        .ok_or_else(|| Error::Message("No value for filename".to_string()))?
        .collect::<Vec<_>>()
        .first()
        .expect("No filename")
        .to_string();

    new_socool_file(filename, cwd)?;
    Ok(())
}

fn new_socool_file(filename: String, working_path: PathBuf) -> Result<(), Error> {
    let path = working_path.join(format!("{filename}.socool"));

    fs::write(path, DEFAULT_SOCOOL).expect("Unable to write file");
    play_file(format!("{filename}.socool"), working_path, Play::Once)?;
    Ok(())
}

const DEFAULT_SOCOOL: &str = indoc! {"
{ f: 311.127, l: 1, g: 1, p: 0 }

thing1 = {
  O[
    (9/4, 7, 1/4, 1/10),
    (9/4, -8, 1/4, -1/10),
    (3/2, 4, 1, 7/8),
    (3/2, -3, 1, -7/8),
    (1/1, 4, 1, 1),
    (1/1, 0, 1, -1),
  ]
  | Seq [
    Fm 1, Fm 9/8, Fm 5/4, Fm 3/2
  ]
}

thing2 = {
  O[
    (1/1, 2, 1, 1),
    (1/1, 0, 1, -1),
  ]
  | Seq [
    Fm 3/4, Fm 1, Fm 4/3, Fm 5/4, Fm 9/8
  ]
  | Seq [Repeat 2, Reverse]
  | FitLength thing1
  
}

main = {
  Overlay [
    thing1,
    thing2
  ]
  | Lm 1/2
}
"};
