use serde_json::{from_reader, to_string, to_string_pretty};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use walkdir::WalkDir;
use weresocool::generation::{
    filename_to_timed_op_1d, filename_to_timed_op_2d, get_min_max_for_path,
    timed_op_1d_to_csv_data_1d, timed_op_2d_to_csv_data_2d, CSVOp,
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
        v: 0,
    };

    let mut min_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        l: 0.0,
        v: 0,
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

    min_max_to_json(min_state, max_state);

    let (min, max) = min_max_from_json();
    dbg!(min.clone());
    dbg!(max.clone());

    filename_to_nn_csv_1d("songs/test_data/nn1.socool".to_string());
}

fn filename_to_nn_json_1d(filename: String) {
    let timed_1d = filename_to_timed_op_1d(filename);
    let csv_1d = timed_op_1d_to_csv_data_1d(timed_1d);

    let pretty = to_string(&csv_1d).unwrap();
    let mut file = File::create("songs/training_data/data_1d.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn filename_to_nn_csv_1d(filename: String) {
    let mut file = File::create("songs/training_data/data_1d.json").unwrap();
    let timed_1d = filename_to_timed_op_1d(filename);
    let csv_1d = timed_op_1d_to_csv_data_1d(timed_1d);

    for op in csv_1d {
        let line = format!(
            "{}, {}, {}, {}, {}, {}, {}\n",
            op.fm, op.fa, op.pm, op.pa, op.g, op.l, op.v as f32
        );
        file.write_all(line.as_bytes()).unwrap();
    }

    //    file.write_all(pretty.as_bytes()).unwrap();
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
