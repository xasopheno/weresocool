#![feature(test)]

extern crate num_rational;
#[macro_use]
extern crate serde_derive;
extern crate indexmap;
extern crate serde_json;
extern crate socool_parser;
extern crate weresocool;
use indexmap::IndexMap;
use serde_json::{from_reader, to_string_pretty};
use socool_parser::parser::*;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use weresocool::{
    generation::parsed_to_render::{generate_waveforms, r_to_f64, sum_all_waveforms},
    instrument::{oscillator::Origin, stereo_waveform::Normalize},
    operations::{NormalForm, Normalize as NormalizeOp},
};

use test::Bencher;

//#![feature(test)]
extern crate test;

type TestTable = IndexMap<String, CompositionHashes>;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let test_table = generate_test_table();
    write_test_table_to_json_file(&test_table);

    let decoded = read_test_table_from_json_file();
//    assert_eq!(test_table, decoded)
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct CompositionHashes {
    op: u64,
    normal_form: u64,
    stereo_waveform: f64,
}

fn write_test_table_to_json_file(test_table: &TestTable) {
    let pretty = to_string_pretty(test_table).unwrap();
    let mut file = File::create("test/hashes.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn read_test_table_from_json_file() -> TestTable {
    let file = File::open("test/hashes.json").unwrap();

    let decoded: TestTable = from_reader(&file).unwrap();
    decoded
}

fn generate_test_table() -> TestTable {
    let mut test_table: TestTable = IndexMap::new();
    let paths = fs::read_dir("./songs/test").unwrap();
    for path in paths {
        let p = path.unwrap().path().into_os_string().into_string().unwrap();
        let composition_hashes = generate_render_hashes(&p);
        test_table.insert(p, composition_hashes);
    }

    test_table.sort_by(|a, _b, c, _d| a.partial_cmp(c).unwrap());
    test_table
}

//#[bench]
//fn bench_1(b: &mut Bencher) {
//    b.iter(|| {
//        1 + 2
//    });
//}

fn generate_render_hashes(p: &String) -> CompositionHashes {
    let parsed = parse_file(p, None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    let op_hash = calculate_hash(main_op);
    let mut normal_form = NormalForm::init();

    main_op.apply_to_normal_form(&mut normal_form);
    let nf_hash = calculate_hash(&normal_form);

    let origin = Origin {
        f: r_to_f64(init.f),
        g: r_to_f64(init.g),
        l: r_to_f64(init.l),
        p: r_to_f64(init.p),
    };

    let vec_wav = generate_waveforms(&origin, normal_form.operations, false);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    let render_hash = sum_vec(result.l_buffer) + sum_vec(result.r_buffer);

    let hashes = CompositionHashes {
        op: op_hash,
        normal_form: nf_hash,
        stereo_waveform: render_hash,
    };

    //    println!("{:#?}", hashes);
    hashes
}

fn sum_vec(vec: Vec<f64>) -> f64 {
    vec.iter().sum()
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
