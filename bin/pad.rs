//use num_rational::{Ratio, Rational64};
//use socool_ast::{
//    ast::{Op, OpOrNf},
//    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
//};
//use socool_parser::parser::*;
//use uuid::Uuid;
//
//use indexmap::IndexMap;
//use std::collections::hash_map::DefaultHasher;
//use std::collections::BTreeSet;
//use std::error::Error;
//use std::hash::{Hash, Hasher};

use num_complex::Complex32;
use num_rational::Rational64;
use socool_ast::operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use socool_parser::parser::*;
use weresocool::analyze::fourier::{vec_f64_to_complex, magnitude, Fourier};

use weresocool::{
    generation::parsed_to_render::{generate_waveforms, sum_all_waveforms},
    instrument::{
        oscillator::{Basis, Oscillator},
        stereo_waveform::{Normalize, StereoWaveform},
    },
};
use std::f64::INFINITY;

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

fn generate_sine_wave(p: &String) -> StereoWaveform {
    let parsed = parse_file(p, None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    let mut normal_form = NormalForm::init();

    main_op.apply_to_normal_form(&mut normal_form, &parsed.table);

    let origin = Basis {
        f: r_to_f64(init.f),
        g: r_to_f64(init.g),
        l: r_to_f64(init.l),
        p: r_to_f64(init.p),
        a: 44100.0,
        d: 44100.0,
    };

    let vec_wav = generate_waveforms(&origin, normal_form.operations, false);
    let mut result = sum_all_waveforms(vec_wav);

    result.normalize();

    result
}

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let a440 = generate_sine_wave(&"songs/test/a_440.socool".to_string())
        .l_buffer
        .clone();
    let start = 44100;
    let buffer_len = 4096;
    let sample_rate = 44100;
    let mut buffer = &a440[start..start + buffer_len];

    let mut fft = vec_f64_to_complex(&mut buffer.to_vec());

    fft.fft();

    #[derive(Debug, PartialOrd, PartialEq)]
    struct Bin {
        mag: f64,
        index: usize,
    }

    let mag = magnitude(&mut fft);
    let mut indexes = vec![];
    let mut min_mag = 150.0;
    for i in 0..mag.len()/2 {
//        if mag[i] > min_mag {
        indexes.push(Bin {
            index: i,
            mag: mag[i]
        });
//        }
    }

//    println!("{:?}", max_mag);
    indexes.sort_by(|a, b| b.mag.partial_cmp(&a.mag).unwrap());

//    println!("{:?}", indexes);
    indexes[0..10].iter().for_each(|bin| {
        if bin.mag > indexes[0].mag * 1.0/2.0 {
            let freq = bin.index as f64 * sample_rate as f64 / buffer_len as f64;
            println!("freq {:?}, magnitude {:?}", freq, bin.mag);
        }
    });
}
