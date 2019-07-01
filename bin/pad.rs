use weresocool::generation::{
    Op4D,
    NormalizerJson,
};
use std::{
    fs::{File},
    io::{BufRead, BufReader} 
};
use serde_json::from_reader;

fn main() {
    let file = File::open("renders/alex.socool.csv").unwrap();
    let normalizer = File::open("songs/normalizers/alex.socool.normalizer")
        .unwrap();
    let normalizer: NormalizerJson = from_reader(&normalizer).unwrap();
    println!("{:?}", normalizer);
    for line in BufReader::new(file).lines() {
        let point = line.unwrap();
        let values: Vec<&str> = point.split(",").collect();

        println!("{:?}", values);
    }
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
