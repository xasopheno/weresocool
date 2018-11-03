extern crate weresocool;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

fn main() {
    let f = File::open("songs/template.socool").unwrap();
    let file = BufReader::new(&f);
    let mut composition = String::new();
    for (num, line) in file.lines().enumerate() {
        let l = line.unwrap();
        let copy_l = l.trim_left();
        if copy_l.starts_with("--") {
            composition.push_str("\n".to_string);
        } else {
            composition.push_str(l);
        }
    }

    write!(composition);
}

