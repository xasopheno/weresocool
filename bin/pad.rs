use num_rational::{BigRational, Rational64};
use socool_ast::{NormalForm, Op, OpOrNf, PointOp};
use socool_parser::float_to_rational::helpers::f32_to_rational;
use weresocool::generation::{NormalizerJson, OpCsv1d};

use std::fs::File;

use csv;
use serde_json::from_reader;

fn main() {
    let file = File::open("renders/template.socool.csv").expect("No Csv File");
    let normalizer_cache =
        File::open("songs/normalizers/template.socool.normalizer").expect("No Normalizer found");
    let normalizer_json: NormalizerJson =
        from_reader(&normalizer_cache).expect("Cant read normalizer_json");
    dbg!(normalizer_json.clone());

    let mut rdr = csv::Reader::from_reader(file);
    let mut nf: Vec<Vec<PointOp>> = vec![vec![]];

    for csv_line in rdr.deserialize() {
        let mut op: OpCsv1d = csv_line.expect("Couldn't convert float into rational");
        op.denormalize(&normalizer_json);
        let v = op.voice;
        let op = op.to_point_op();
        if v < nf.len() {
            nf[v].push(op);
        } else {
            for _ in 0..(v - nf.len() + 1) {
                nf.push(vec![]);
            }
            nf[v].push(op);
        };
    }
    println!("nf {:?}", nf);
    println!("done");
}
