extern crate num_rational;
extern crate socool_parser;
extern crate weresocool;
use num_rational::Rational64;
use socool_parser::ast::Op::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let mut seq = vec![1.0, 2.0, 3.0].into_iter().cycle();
    let mut counter = 0.0;
    let inc = 0.5;
    let mut current = seq.next().unwrap();
    while true {
        if counter >= current {
            counter = 0.0;
            current = seq.next().unwrap()
        }
        println!("{:?}", current);
        counter += inc;
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
