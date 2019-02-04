extern crate indexmap;
extern crate num_rational;
extern crate socool_ast;
extern crate socool_parser;
extern crate uuid;
use num_rational::{Ratio, Rational64};
use socool_ast::{
    ast::{Op, OpOrNf},
    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
};
use socool_parser::parser::*;
use uuid::Uuid;

use indexmap::IndexMap;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::error::Error;
use std::hash::{Hash, Hasher};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let uuid = new_uuid();
    println!("{:?}", uuid);
}

fn new_uuid() -> Uuid {
    Uuid::new_v4()
}
