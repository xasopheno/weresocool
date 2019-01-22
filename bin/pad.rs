extern crate num_rational;
#[macro_use]
extern crate serde_derive;
extern crate indexmap;
extern crate serde_json;
extern crate socool_parser;
extern crate weresocool;
use fs::write;
use indexmap::IndexMap;
use num_rational::Rational64;
use serde_json::{from_reader, to_string_pretty, to_writer};
use socool_parser::ast::Op::*;
use socool_parser::parser::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use weresocool::{
    generation::parsed_to_render::{generate_waveforms, r_to_f64, render, sum_all_waveforms},
    instrument::{
        oscillator::Origin,
        stereo_waveform::{Normalize, StereoWaveform},
    },
    operations::{NormalForm, Normalize as NormalizeOp},
};

type TestTable = IndexMap<String, CompositionHashes>;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let mut hm: TestTable = IndexMap::new();
    let paths = fs::read_dir("./songs/test").unwrap();
    for path in paths {
        let p = path.unwrap().path().into_os_string().into_string().unwrap();
        //        if p.ends_with("pan_test.socool") {
        let composition_hashes = generate_render_hashes(&p);
        hm.insert(p, composition_hashes);
    }

    hm.sort_by(|a, _b, c, _d| a.partial_cmp(c).unwrap());

    {
        let pretty = to_string_pretty(&hm).unwrap();
        let mut file = File::create("test_hashes.json").unwrap();
        file.write_all(pretty.as_bytes());
    }

    let mut file = File::open("test_hashes.json").unwrap();

    let decoded: TestTable = from_reader(&file).unwrap();
    println!("{:#?}", decoded);
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct CompositionHashes {
    op: u64,
    normal_form: u64,
    stereo_waveform: f64,
}

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
