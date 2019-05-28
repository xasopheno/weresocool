use serde_json::{from_reader, to_string, to_string_pretty};
use socool_ast::{Op::*, OpOrNf, OpOrNf::*};
use socool_parser::f32_to_rational;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use walkdir::WalkDir;
use weresocool::generation::{
    filename_to_timed_op_1d, filename_to_timed_op_2d, get_min_max_for_path,
    timed_op_1d_to_csv_data_1d, timed_op_2d_to_csv_data_2d, CSVOp, Normalizer,
};

fn main() {
    println!("Hello WereSoCool Scratch Pad");

    let mut max_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        l: 0.0,
        v: 0.0,
    };

    let mut min_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        l: 0.0,
        v: 0.0,
    };

    let mut max_seq_length = 0;

    for entry in WalkDir::new("./songs/training_data")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.path().to_string_lossy();
        println!("{:?}", f_name);

        if f_name.ends_with(".socool") {
            let (song_max, song_min, n_voices) = get_min_max_for_path(f_name.to_string());
            max_state = CSVOp {
                fm: max_state.fm.max(song_max.fm),
                fa: max_state.fa.max(song_max.fa),
                pm: max_state.pm.max(song_max.pm),
                pa: max_state.pa.max(song_max.pa),
                g: max_state.g.max(song_max.g),
                l: max_state.l.max(song_max.l),
                v: max_state.v.max(song_max.v),
            };
            min_state = CSVOp {
                fm: min_state.fm.min(song_min.fm),
                fa: min_state.fa.min(song_min.fa),
                pm: min_state.pm.min(song_min.pm),
                pa: min_state.pa.min(song_min.pa),
                g: min_state.g.min(song_min.g),
                l: min_state.l.min(song_min.l),
                v: min_state.v.min(song_min.v),
            };

            max_seq_length = max_seq_length.max(n_voices);

            println!("{:#?}", n_voices);
        }
    }

    //    min_max_to_json(min_state, max_state);

    let (min, max) = min_max_from_json();
    dbg!(min.clone());
    dbg!(max.clone());

    let normalizer = Normalizer::from_min_max(min, max);

    //    filename_to_nn_csv_1d("songs/test_data/nn1.socool".to_string(), normalizer);
    csv_1d_to_normalform();
}

fn csv_to_point_op(csv: Vec<str>) {
    let fm = csv[0].clone();
    let fa = csv[1].clone();
    let pm = csv[2].clone();
    let pa = csv[3].clone();
    let g = csv[4].clone();
    let l = csv[5].clone();
    let v = csv[6].clone();

    dbg!(v);

    let op = Op(Compose {
        operations: vec![
            Op(TransposeM {
                m: f32_to_rational(fm),
            }),
            Op(TransposeA {
                a: f32_to_rational(fa),
            }),
            Op(PanM {
                m: f32_to_rational(pm),
            }),
            Op(PanA {
                a: f32_to_rational(pa),
            }),
            Op(Gain {
                m: f32_to_rational(g),
            }),
            Op(Length {
                m: f32_to_rational(l),
            }),
        ],
    });
    dbg!(op);
}

fn csv_1d_to_normalform() {
    let f = File::open("songs/training_data/data_1d.csv").unwrap();
    let file = BufReader::new(&f);
    for (num, line) in file.lines().enumerate() {
        let l = line.unwrap();
        let split = l.split(",");
        let values: Vec<&str> = split.collect();
        println!("{:?}", values);
    }
}

fn filename_to_nn_json_1d(filename: String) {
    let timed_1d = filename_to_timed_op_1d(filename);
    let csv_1d = timed_op_1d_to_csv_data_1d(timed_1d);

    let pretty = to_string(&csv_1d).unwrap();
    let mut file = File::create("songs/training_data/data_1d.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn filename_to_nn_csv_1d(filename: String, normalizer: Normalizer) {
    let mut file = File::create("songs/training_data/data_1d.csv").unwrap();
    let timed_1d = filename_to_timed_op_1d(filename);
    let csv_1d = timed_op_1d_to_csv_data_1d(timed_1d);

    for mut op in csv_1d {
        op.normalize(&normalizer);
        let line = format!(
            "{}, {}, {}, {}, {}, {}, {}\n",
            op.fm, op.fa, op.pm, op.pa, op.g, op.l, op.v
        );
        file.write_all(line.as_bytes()).unwrap();
    }
}

fn filename_to_nn_json_2d(filename: String) {
    let timed_2d = filename_to_timed_op_2d(filename);
    let csv_2d = timed_op_2d_to_csv_data_2d(timed_2d);

    let pretty = to_string(&csv_2d).unwrap();
    let mut file = File::create("songs/training_data/data.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn min_max_to_json(min: CSVOp, max: CSVOp) {
    let pretty = to_string_pretty(&min).unwrap();
    let mut file = File::create("songs/training_data/min.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();

    let pretty = to_string_pretty(&max).unwrap();
    let mut file = File::create("songs/training_data/max.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn min_max_from_json() -> (CSVOp, CSVOp) {
    let path = "songs/training_data/min.json";
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let min: CSVOp = serde_json::from_reader(reader).unwrap();

    let path = "songs/training_data/max.json";
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let max: CSVOp = serde_json::from_reader(reader).unwrap();

    (min, max)
}

#[test]
fn test_test() {
    assert!(true, true);
}
