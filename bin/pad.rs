//use num_rational::{Ratio, Rational64};
//use socool_ast::{
//    ast::{Op, OpOrNf},
//    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
//};
//use socool_parser::*;
//use uuid::Uuid;
//
//use indexmap::IndexMap;
//use std::collections::hash_map::DefaultHasher;
//use std::collections::BTreeSet;
//use std::error::Error;
//use std::hash::{Hash, Hasher};
//use num_complex::Complex32;
//use num_rational::{Ratio, Rational64};
//use portaudio as pa;
//use socool_ast::{
//    ast::{Op::*, OpOrNf::*},
//    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
//};
//use socool_parser::{float_to_rational::helpers::f32_to_rational, parser::*};
//use weresocool::portaudio::setup_portaudio_output;
//
//use weresocool::analyze::fourier::{magnitude, vec_f64_to_complex, Fourier};
//
//use socool_ast::Op::{Overlay, TransposeM};
//use std::f64::INFINITY;
//use weresocool::generation::parsed_to_render::render;
//use weresocool::{
//    generation::parsed_to_render::{generate_waveforms, sum_all_waveforms},
//    instrument::{
//        Basis, Oscillator,
//        Normalize, StereoWaveform,
//    },
//};

//use num::Integer;
//use rand::Rng;
//use serde::Deserialize;
//use serde_json::from_str;
//use socool_ast::{OpOrNf, OpOrNfTable};
//use std::collections::HashMap;
//use std::fs::File;
//use std::io::Read;
//use std::path::Path;

use weresocool::generation::{EventType, Op4D};

pub struct Op4D {
    pub t: f64,
    pub event_type: EventType,
    pub voice: usize,
    pub event: usize,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub l: f64,
}
fn main() {
    println!("Hello Scratch Pad");
    let mut max_state = Op4D {
        t: 0.0,
        event: 0,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
    };

    let mut min_state = Op4D {
        t: 0.0,
        event: 0,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
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
            max_state = Op4D {
                x: max_state.x.max(song_max.fm),
                //                fa: max_state.fa.max(song_max.fa),
                //                pm: max_state.pm.max(song_max.pm),
                //                pa: max_state.pa.max(song_max.pa),
                //                g: max_state.g.max(song_max.g),
                //                l: max_state.l.max(song_max.l),
                //                v: max_state.v.max(song_max.v),
            };
            //            min_state = Op4D {
            //                fm: min_state.fm.min(song_min.fm),
            //                fa: min_state.fa.min(song_min.fa),
            //                pm: min_state.pm.min(song_min.pm),
            //                pa: min_state.pa.min(song_min.pa),
            //                g: min_state.g.min(song_min.g),
            //                l: min_state.l.min(song_min.l),
            //                v: min_state.v.min(song_min.v),
            //            };

            max_seq_length = max_seq_length.max(n_voices);

            println!("{:#?}", n_voices);
        }
    }
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
