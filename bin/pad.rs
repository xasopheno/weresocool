use weresocool::generation::json::Op4D;
use std::{
    fs::File,
    io::{BufRead, BufReader} 
};

fn main() {
    let file = File::open("renders/alex.socool.csv").unwrap();
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
