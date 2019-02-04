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

    table.set(&"name".to_string(), payload);

    println!("{:?}", table);
}

fn new_uuid() -> Uuid {
    Uuid::new_v4()
}

type Table = IndexMap<String, Get>;

pub trait API {
    //        fn get(&self, name: &String) -> Result<OpOrNf, Box<Error>>;
    fn set(&mut self, name: &String, payload: Payload) -> Result<String, Box<Error>>;
}

impl API for Table {
    //    fn get(&self, name: &String) -> Result<OpOrNf, Box<Error>> {
    //        Result::
    //    }
    fn set(&mut self, name: &String, payload: Payload) -> Result<String, Box<Error>> {
        let uuid = new_uuid().to_string();
        self.insert(uuid.to_string(), Get::Payload(payload));
        //        for (name, entry) in self.iter() {
        //            match entry {
        //                Get::Pointer(_) => {}
        //                Get::Payload(value) => {
        //                    if value.context.filename == payload.context.filename
        //                        && value.context.scope == payload.context.scope {
        //
        //                    }
        //                }
        //            };
        //        }
        //        result
        Ok(uuid.to_string())
    }
}

#[derive(Debug)]
pub enum Get {
     Pointer(Uuid),
    Payload(Payload),
}

#[derive(Debug)]
pub struct Payload {
    import_name: String,
    context: Context,
    op_or_nf: OpOrNf,
}

#[derive(Debug)]
enum Scope {
    Global,
    Scope(Uuid),
}

#[derive(Debug)]
struct Context {
    filename: String,
    scope: Scope,
}

#[derive(Debug)]
struct Function {
    passed_in: Vec<OpOrNf>,
    uuid: Uuid,
}
