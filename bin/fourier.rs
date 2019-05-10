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
use num_rational::{Ratio, Rational64};
use portaudio as pa;
use socool_ast::{
    ast::{Op::*, OpOrNf::*},
    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
};
use socool_parser::{float_to_rational::helpers::f32_to_rational, parser::*};
use weresocool::portaudio_setup::output::setup_portaudio_output;

use weresocool::analyze::fourier::{magnitude, vec_f64_to_complex, Fourier};

use socool_ast::ast::Op::{Overlay, TransposeM};
use std::f64::INFINITY;
use weresocool::generation::parsed_to_render::render;
use weresocool::{
    generation::parsed_to_render::{generate_waveforms, sum_all_waveforms},
    instrument::{
        oscillator::{Basis, Oscillator},
        stereo_waveform::{Normalize, StereoWaveform},
    },
};

use num::Integer;
use rand::Rng;
use serde::Deserialize;
use serde_json::from_str;
use socool_ast::ast::{OpOrNf, OpOrNfTable};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type StockValue = HashMap<String, HashMap<String, String>>;

fn read_json(path: &str) -> StockValue {
    let json_file_path = Path::new(path);
    let json_file = File::open(json_file_path).expect("file not found");
    let deserialized: StockValue =
        serde_json::from_reader(json_file).expect("error while reading json");
    deserialized
}

fn decimal_to_rational(mut n: f64) -> [isize; 2] {
    //Based on Farey sequences
    assert!(n.is_finite());
    let flag_neg = n < 0.0;
    if flag_neg {
        n = n * (-1.0)
    }
    if n < std::f64::MIN_POSITIVE {
        return [0, 1];
    }
    if (n - n.round()).abs() < std::f64::EPSILON {
        return [n.round() as isize, 1];
    }
    let mut a: isize = 0;
    let mut b: isize = 1;
    let mut c: isize = n.ceil() as isize;
    let mut d: isize = 1;
    let aux1 = isize::max_value() / 2;
    while c < aux1 && d < aux1 {
        let aux2: f64 = (a as f64 + c as f64) / (b as f64 + d as f64);
        if (n - aux2).abs() < std::f64::EPSILON {
            break;
        }
        if n > aux2 {
            a = a + c;
            b = b + d;
        } else {
            c = a + c;
            d = b + d;
        }
    }
    // Make sure that the fraction is irreducible
    let gcd = (a + c).gcd(&(b + d));
    if flag_neg {
        [-(a + c) / gcd, (b + d) / gcd]
    } else {
        [(a + c) / gcd, (b + d) / gcd]
    }
}

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

fn generate_sine_wave(p: &String) -> (StereoWaveform, Basis, NormalForm) {
    let parsed = parse_file(p, None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    let mut normal_form = NormalForm::init();

    main_op.apply_to_normal_form(&mut normal_form, &parsed.table);

    let basis = Basis {
        f: r_to_f64(init.f),
        g: r_to_f64(init.g),
        l: r_to_f64(init.l),
        p: r_to_f64(init.p),
        a: 44100.0,
        d: 44100.0,
    };

    let vec_wav = generate_waveforms(&basis, normal_form.clone().operations, false);
    let mut result = sum_all_waveforms(vec_wav);

    result.normalize();

    (result, basis, normal_form)
}

fn vec_vec_freq_to_overlay(vecs: Vec<Vec<f64>>) -> OpOrNf {
    let mut overlay_vec = vec![];
    let num_vecs = vecs.len();
    for (n, vec) in vecs.iter().enumerate() {
        let mut sequence_vec = vec![];
        let denom = vec[0].clone();
        for freq in vec {
            let m = decimal_to_rational((*freq as f64 / denom));
            let mut m = Rational64::new(m[0] as i64, m[1] as i64);
            if m < Rational64::new(1, 1) {
                //                m *= Rational64::new(0, 1)
            }
            //            else {
            //                m *= Rational64::new(1, 2)
            //            }
            sequence_vec.push(Op(TransposeM { m }));
        }
        let switch: i64 = if n % 2 == 0 { 1 } else { -1 };
        overlay_vec.push(Op(Compose {
            operations: vec![
                Op(Sequence {
                    operations: sequence_vec,
                }),
                Op(PanA {
                    a: Rational64::new((switch * (n as i64 + 1)), num_vecs as i64),
                }),
            ],
        }));
    }

    Op(Overlay {
        operations: overlay_vec,
    })
}

fn overlay_to_normal_form(overlay: OpOrNf) -> NormalForm {
    let mut nf = NormalForm::init();

    let mut op_or_nf_table = OpOrNfTable::new();

    overlay.apply_to_normal_form(&mut nf, &op_or_nf_table);

    nf
}

fn vec_bin_to_normalform(bins: Vec<Bin>) -> NormalForm {
    let mut overlay_vec = vec![];
    for bin in bins {
        let freq = bin.to_freq();
        dbg!(freq);
        let m = decimal_to_rational(freq / 100.0);
        let m = Rational64::new(m[0] as i64, m[1] as i64);
        println!("{:?}", m);
        overlay_vec.push(Op(TransposeM { m }));
    }

    let overlay = Op(Overlay {
        operations: overlay_vec,
    });

    let mut nf = NormalForm::init();

    let mut op_or_nf_table = OpOrNfTable::new();

    overlay.apply_to_normal_form(&mut nf, &op_or_nf_table);

    nf
}

#[derive(Debug, Clone)]
struct Bin {
    mag: f64,
    index: usize,
}

impl Bin {
    fn to_freq(&self) -> f64 {
        let buffer_len = 10000;
        let sample_rate = 44100;
        self.index as f64 * sample_rate as f64 / buffer_len as f64
    }
}

fn fft_stuff() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let (sound, basis, mut nf1) = generate_sine_wave(&"songs/test/a_440.socool".to_string());
    let a440 = sound.l_buffer.clone();
    let start = 5000;
    let buffer_len = 2048;
    let sample_rate = 44100;
    let mut buffer = &a440[start..start + buffer_len];

    let mut fft = vec_f64_to_complex(&mut buffer.to_vec());

    fft.fft();

    #[derive(Debug, PartialOrd, PartialEq)]
        let mag = magnitude(&mut fft);
    let mut indexes = vec![];
    //    let mut min_mag = 150.0;
    for i in 0..mag.len() / 2 {
        //        if mag[i] > min_mag {
        indexes.push(Bin {
            index: i,
            mag: mag[i],
        });
        //        }
    }

    //    println!("{:?}", max_mag);
    indexes.sort_by(|a, b| b.mag.partial_cmp(&a.mag).unwrap());
}

fn main() -> Result<(), pa::Error> {
    //    println!("{:?}", indexes);
    //    indexes[0..10].iter().for_each(|bin| {
    //        if bin.mag > indexes[0].mag * 1.0/2.0 {
    //            let freq = bin.index as f64 * sample_rate as f64 / buffer_len as f64;
    //            println!("freq {:?}, magnitude {:?}", freq, bin.mag);
    //        }
    //    });

    let mut freqs = vec![];
    for filename in &[
        //        "f.json",
        "msft.json",
        "goog.json",
        "nvda.json",
        "tsla.json",
    ] {
        let mut stock_seq = vec![];
        let json = read_json(filename);

        for (date, data) in json {
            for (key, value) in data {
                if key.starts_with("5.") {
                    let freq: f32 = value.parse().unwrap();
                    stock_seq.push(freq as f64)
                }
            }
        }
        freqs.push(stock_seq);
    }

    //    let freqs = vec![
    //        vec![
    //            203.0, 300.0, 340.0
    //        ],
    //        vec![
    //            100.0, 200.0, 300.0,
    //        ]
    //    ];

    let basis = Basis {
        f: 500.0,
        g: 1.0,
        l: 3.0,
        p: 0.0,
        a: 44100.0,
        d: 44100.0,
    };

    let overlay = vec_vec_freq_to_overlay(freqs);
    let mut nf = overlay_to_normal_form(overlay);

    let parsed_table = OpOrNfTable::new();

    let composition = render(&basis, &nf, &parsed_table);

    let pa = pa::PortAudio::new()?;

    let mut output_stream = setup_portaudio_output(composition, &pa)?;

    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;

    Ok(())
}

#[test]
fn test_decimal_to_fraction() {
    // Test the function with 1_000_000 random decimal numbers
    let mut rng = rand::thread_rng();
    for _i in 1..1_000_000 {
        let number = rng.gen::<f64>();
        let result = decimal_to_rational(number);
        assert!((number - (result[0] as f64) / (result[1] as f64)).abs() < std::f64::EPSILON);
        assert!(result[0].gcd(&result[1]) == 1);
    }
}
