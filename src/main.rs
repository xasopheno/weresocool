#[macro_use]
extern crate lalrpop_util;
extern crate colored;
use colored::*;
pub mod ast;
use crate::ast::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

lalrpop_mod!(pub socool);

pub type ParseTable = HashMap<String, Op>;

pub struct ParsedComposition {
    init: Init,
    table: ParseTable,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename;
    if args.len() == 2 {
        filename = &args[1];
    } else {
        println!("\n{}\n", "Forgot to pass in a filename.".red().bold());
        panic!("Wrong number of args")

    }

    let parsed = parse_file(filename);

    for (key, val) in parsed.table.iter() {
        println!("\n Name: {:?} op: {:?}", key, val);
    }

    println!("\n Main: {:?}", parsed.table.get("main").unwrap());
}

fn parse_file(filename: &String) -> ParsedComposition {
    let mut table = make_table();
    let mut f = File::open(filename).expect("File not found");

    let mut composition = String::new();
    f.read_to_string(&mut composition)
        .expect("Something went wrong reading the file");

    let init = socool::SoCoolParser::new()
        .parse(&mut table, &composition)
        .unwrap();

    ParsedComposition { init, table }
}

fn make_table() -> ParseTable {
    HashMap::new()
}

#[cfg(test)]
mod test;
