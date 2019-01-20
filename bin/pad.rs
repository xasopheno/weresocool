extern crate num_rational;
extern crate socool_parser;
extern crate weresocool;
use num_rational::Rational64;
use socool_parser::ast::Op::*;
use socool_parser::parser::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
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

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let paths = fs::read_dir("./songs/test").unwrap();
    for path in paths {
        let p = path.unwrap().path();
        if p.ends_with("pan_test.socool") {
            generate_render_hashes(p);
        }
    }
}

fn generate_render_hashes(p: PathBuf) {
    println!("{:?}", p);
    let parsed = parse_file(&p.into_os_string().into_string().unwrap(), None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    let op_hash = calculate_hash(main_op);
    println!("{}", op_hash);
//            assert_eq!(op_hash, 11366878093498661911);
    let mut normal_form = NormalForm::init();

    println!("\nGenerating Composition ");
    main_op.apply_to_normal_form(&mut normal_form);
    let nf_hash = calculate_hash(&normal_form);
    println!("{}", nf_hash);
//            assert_eq!(nf_hash, 17888512810530479489);

    let origin = Origin {
        f: r_to_f64(init.f),
        g: r_to_f64(init.g),
        l: r_to_f64(init.l),
        p: r_to_f64(init.p),
    };

    let vec_wav = generate_waveforms(&origin, normal_form.operations);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    let render_hash = sum_vec(result.l_buffer) + sum_vec(result.r_buffer);

    println!("{:?}", render_hash);
//            assert_eq!(render_hash, -0.25227999105205157)
}

fn sum_vec(vec: Vec<f64>) -> f64 {
    vec.iter().sum()
}

fn get_file_hash(p: PathBuf) -> u64 {
    let parsed = parse_file(&p.into_os_string().into_string().unwrap(), None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    calculate_hash(main_op)
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
