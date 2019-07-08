use socool_ast::{NormalForm, Op, OpOrNf};
use weresocool::generation::{NormalizerJson, OpCsv1d};

use std::fs::File;

use csv;
use serde_json::from_reader;

fn main() {
    //let file = File::open("renders/alex.socool.csv").unwrap();
    //let normalizer_cache = File::open("songs/normalizers/alex.socool.normalizer").unwrap();
    //let normalizer_json: NormalizerJson = from_reader(&normalizer_cache).unwrap();
    //dbg!(normalizer_json.clone());

    //let mut rdr = csv::Reader::from_reader(1hing {

    #[derive(Debug, Clone)]
    struct Thing {
        val: u32,
        voice: usize,
    };

    let things = vec![
        Thing { val: 1, voice: 0 },
        Thing { val: 2, voice: 1 },
        Thing { val: 3, voice: 0 },
        Thing { val: 4, voice: 0 },
        Thing { val: 5, voice: 2 },
    ];

    let mut nf: Vec<Vec<Thing>> = vec![vec![]];

    for thing in things {
        let v = thing.voice;
        if v < nf.len() {
            nf[v].push(thing);
        } else {
            for i in 0..(v - nf.len() + 1) {
                nf.push(vec![]);
            }
            nf[v].push(thing);
        }
    }

    dbg!(nf);

    //for csv_line in rdr.deserialize() {
    //let mut op: OpCsv1d = csv_line.unwrap();
    //op.denormalize(&normalizer_json);

    //println!("{:?}", op);
    //}
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
