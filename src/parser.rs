lalrpop_mod!(pub socool);

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use crate::ast::*;

pub type ParseTable = HashMap<String, Op>;

pub struct ParsedComposition {
    pub init: Init,
    pub table: ParseTable,
}

pub fn parse_file(filename: &String) -> ParsedComposition {
    let mut table = HashMap::new();
    let mut f = File::open(filename).expect("File not found");

    let mut composition = String::new();
    f.read_to_string(&mut composition)
        .expect("Something went wrong reading the file");

    let init = socool::SoCoolParser::new()
        .parse(&mut table, &composition)
        .unwrap();

    ParsedComposition { init, table }
}


