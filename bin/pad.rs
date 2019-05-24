use serde_json::{from_reader, to_string_pretty};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use walkdir::WalkDir;
use weresocool::generation::{get_min_max_for_path, CSVOp};

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

    println!(
        "MAX {:#?}\nMIN {:#?}\nMAX_SEQ_LENGTH {:?}",
        max_state, min_state, max_seq_length
    );

    min_max_to_json(min_state, max_state);

    let (min, max) = min_max_from_json();
    dbg!(min);
    dbg!(max);
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
