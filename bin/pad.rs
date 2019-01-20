extern crate num_rational;
extern crate socool_parser;
extern crate weresocool;
use num_rational::Rational64;
use socool_parser::ast::Op::*;
use socool_parser::parser::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use weresocool::operations::{Normalize, NormalForm};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let paths = fs::read_dir("./songs/test").unwrap();
    for path in paths {
        let p = path.unwrap().path();
        if p.ends_with("pan_test.socool") {
            println!("{:?}", p);
            let parsed = parse_file(&p.into_os_string().into_string().unwrap(), None);
            let main_op = parsed.table.get("main").unwrap();
            let init = parsed.init;
            let op_hash = calculate_hash(main_op);
            println!("{}", op_hash);
            assert_eq!(op_hash, 11366878093498661911);
            let mut normal_form = NormalForm::init();

            println!("\nGenerating Composition ");
            main_op.apply_to_normal_form(&mut normal_form);
            let nf_hash = calculate_hash(&normal_form);
            println!("{}", nf_hash);
        }
    }
}

fn get_file_hash(p: PathBuf) -> u64 {
    let parsed = parse_file(&p.into_os_string().into_string().unwrap(), None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    calculate_hash(main_op)
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
