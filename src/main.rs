use std::collections::HashMap;
#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub socool); // synthesized by LALRPOP
pub mod ast;
use crate::ast::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut table = make_table();

    let filename = "working.socool";

    let mut f = File::open(filename).expect("file not found");

    let mut composition = String::new();
    f.read_to_string(&mut composition)
        .expect("something went wrong reading the file");

    println!(
        "\n Settings: {:?}",
        socool::SoCoolParser::new()
            .parse(&mut table, &composition)
            .unwrap()
    );

    for (key, val) in table.iter() {
        println!("\n name: {:?} op: {:?}", key, val);
    }

    println!("\n Main: {:?}", table.get("main").unwrap());
}

fn make_table() -> HashMap<String, Op> {
    let table: HashMap<String, Op> = HashMap::new();
    table
}

#[cfg(test)]
mod test;

