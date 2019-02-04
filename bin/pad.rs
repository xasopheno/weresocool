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

    let mut table = Table::new();

    let payload = Payload {
        context: Context {
            filename: "test.socool".to_string(),
            scope: Scope::Global,
        },
        import_name: "asdf".to_string(),
        op_or_nf: OpOrNf::Op(Op::AsIs),
    };

    let u1 = table.set(&"name_of_it".to_string(), payload.clone());

    let u2 = table.set(&"name_of_it".to_string(), payload.clone());
    let u3 = table.set(&"name_of_it".to_string(), payload.clone());
    let u3 = table.set(&"name_of_it".to_string(), payload);

    println!("u1 {:?}\nu2 {:?}\nu3 {:?}", u1, u2, u3);
    println!("{:?}\n", table);
}

fn new_uuid() -> Uuid {
    Uuid::new_v4()
}

type Table = IndexMap<String, Get>;

pub trait API {
    //        fn get(&self, name: &String) -> Result<OpOrNf, Box<Error>>;
    fn set(&mut self, name: &String, payload: Payload) -> Result<(String), Box<Error>>;
}

impl API for Table {
    //    fn get(&self, name: &String) -> Result<OpOrNf, Box<Error>> {
    //        Result::
    //    }
    fn set(&mut self, name: &String, payload: Payload) -> Result<(String), Box<Error>> {
        let mut result = name.clone();
        for (look_up_name, entry) in self.clone().iter() {
            match entry {
                Get::Pointer(_) => {}
                Get::Payload(value) => {
                    if value.context.filename == payload.context.filename
                        && value.context.scope == payload.context.scope
                    {
                        let uuid = new_uuid().to_string();
                        self.insert(uuid.clone(), Get::Pointer(look_up_name.to_string()));
                        result = uuid
                    }
                }
            };
        }
        if result == name.to_string() {
            self.insert(name.to_string(), Get::Payload(payload));
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Get {
    Pointer(String),
    Payload(Payload),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Payload {
    import_name: String,
    context: Context,
    op_or_nf: OpOrNf,
}

#[derive(Debug, Clone, PartialEq, Hash)]
enum Scope {
    Global,
    Scope(Uuid),
}

#[derive(Debug, Clone, PartialEq, Hash)]
struct Context {
    filename: String,
    scope: Scope,
}

#[derive(Debug, Clone, PartialEq, Hash)]
struct Function {
    passed_in: Vec<OpOrNf>,
    uuid: Uuid,
}
