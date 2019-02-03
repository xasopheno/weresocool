extern crate socool_parser;
extern crate num_rational;
extern crate socool_ast;
extern crate indexmap;
use num_rational::{Ratio, Rational64};
use socool_parser::parser::*;
use socool_ast::{
    ast::{
        OpOrNf,
    },
    operations::{
        NormalForm,
        Normalize as NormalizeOp,
        PointOp,
    },
};

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use indexmap::IndexMap;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");


}

type UUID = String;
type Table = IndexMap<String, Get>;

enum Scope {
    Global,
    Scope(UUID),
}

struct Context {
    filename: String,
    scope: Scope
}

struct Let {
    context: Context,
    import_name: String,
}

struct Function {
    passed_in: Vec<OpOrNf>,
    uuid: UUID
}

enum Get {
    Pointer(string),
    Let(Let),
}
