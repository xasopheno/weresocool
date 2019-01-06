extern crate num_rational;
extern crate socool_parser;
extern crate weresocool;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use weresocool::operations::PointOp;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let hash = calculate_hash(&vec![PointOp::init(), PointOp::init()]);
    println!("{:?}", hash);
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
