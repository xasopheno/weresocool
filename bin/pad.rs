use weresocool::generation::{
    OpCsv1d,
    Op4D,
    NormalizerJson,
};
use std::{
    fs::{File},
    io::{BufRead, BufReader},
};
use serde_json::from_reader;
use serde::Deserialize;
use csv;

fn main() {
    let file = File::open("renders/alex.socool.csv").unwrap();
    let normalizer = File::open("songs/normalizers/alex.socool.normalizer")
        .unwrap();
    let normalizer: NormalizerJson = from_reader(&normalizer).unwrap();
    println!("{:?}", normalizer);
    
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let op: OpCsv1d = result.unwrap();

        println!("{:?}", op);
    }
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
