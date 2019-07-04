use socool_ast::{NormalForm, Op, OpOrNf};
use weresocool::generation::{NormalizerJson, OpCsv1d};

use std::fs::File;

use csv;
use serde_json::from_reader;

fn main() {
    let file = File::open("renders/alex.socool.csv").unwrap();
    let normalizer_cache = File::open("songs/normalizers/alex.socool.normalizer").unwrap();
    let normalizer_json: NormalizerJson = from_reader(&normalizer_cache).unwrap();
    dbg!(normalizer_json.clone());

    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let mut op: OpCsv1d = result.unwrap();
        op.denormalize(&normalizer_json);

        if op.frequency > 3.962 {
            println!("{:?}", op.frequency);
        }
    }
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
