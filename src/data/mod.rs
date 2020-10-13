use crate::generation::{RenderReturn, RenderType};
use crate::interpretable::{InputType::Filename, Interpretable};
use num_rational::Rational64;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use walkdir::WalkDir;
use weresocool_ast::{NormalForm, OscType, PointOp};
use weresocool_error::Error;
use weresocool_parser::float_to_rational::helpers::f32_to_rational;
use weresocool_shared::helpers::r_to_f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataOp {
    pub fm: Rational64,
    pub fa: Rational64,
    pub g: Rational64,
    pub l: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub osc_type: Rational64,
}

#[derive(Debug, Clone)]
pub struct Normalizer {
    pub fm: MinMax,
    pub fa: MinMax,
    pub g: MinMax,
    pub l: MinMax,
    pub pm: MinMax,
    pub pa: MinMax,
}

impl Normalizer {
    pub const fn from_min_max(min: DataOp, max: DataOp) -> Self {
        Self {
            fm: MinMax {
                min: min.fm,
                max: max.fm,
            },
            fa: MinMax {
                min: min.fm,
                max: max.fm,
            },
            pm: MinMax {
                min: min.pm,
                max: max.pm,
            },
            pa: MinMax {
                min: min.pa,
                max: max.pa,
            },
            g: MinMax {
                min: min.g,
                max: max.g,
            },
            l: MinMax {
                min: min.l,
                max: max.l,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct MinMax {
    pub min: Rational64,
    pub max: Rational64,
}

fn denormalize_value(val: Rational64, min: Rational64, max: Rational64) -> Rational64 {
    val * (max - min) + min
}
fn normalize_value(value: Rational64, min: Rational64, max: Rational64) -> Rational64 {
    if max == min {
        // panic!("max == min in normalize_value");
        return value;
    }
    (value - min) / (max - min)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NNOp {
    pub fm: f64,
    pub fa: f64,
    pub g: f64,
    pub l: f64,
    pub pm: f64,
    pub pa: f64,
    pub osc_type: f64,
}

fn number_to_string(n: f64) -> String {
    format!("{:.32}", n)
}

impl NNOp {
    pub fn write_to_file(self, file: &mut std::fs::File) -> Result<(), Error> {
        file.write(
            format!(
                // "{}, {}, {}, {}, {}, {}, {}\n",
                "{}, {}\n",
                number_to_string(self.fm),
                // number_to_string(self.fa),
                // number_to_string(self.pm),
                // number_to_string(self.pa),
                number_to_string(self.l),
                // number_to_string(self.g),
                // number_to_string(self.osc_type),
            )
            .as_bytes(),
        )?;
        Ok(())
    }
}

impl DataOp {
    pub fn from_vec_f64_string(vec: Vec<String>) -> Self {
        Self {
            fm: f32_to_rational(vec[0].to_owned()),
            l: f32_to_rational(vec[1].to_owned()),
            fa: Rational64::from_integer(0),
            pm: Rational64::from_integer(1),
            pa: Rational64::from_integer(0),
            g: Rational64::from_integer(1),
            osc_type: Rational64::from_integer(0),
        }
    }

    pub fn from_vec_f64_string_bak(vec: Vec<String>) -> Self {
        Self {
            fm: f32_to_rational(vec[0].to_owned()),
            fa: f32_to_rational(vec[1].to_owned()),
            pm: f32_to_rational(vec[2].to_owned()),
            pa: f32_to_rational(vec[3].to_owned()),
            l: f32_to_rational(vec[4].to_owned()),
            g: f32_to_rational(vec[5].to_owned()),
            osc_type: f32_to_rational(vec[6].to_owned()),
        }
    }

    pub fn to_point_op(self) -> PointOp {
        let osc_type = if self.osc_type > Rational64::new(1, 2) {
            OscType::Noise
        } else {
            OscType::Sine { pow: None }
        };

        let mut point_op = PointOp::init_silent();

        point_op.fm = self.fm;
        point_op.fa = self.fa;
        point_op.g = self.g;
        point_op.l = self.l;
        point_op.pm = self.pm;
        point_op.pa = self.pa;
        point_op.osc_type = osc_type;

        point_op
    }
    pub fn to_nnop(self) -> NNOp {
        NNOp {
            fm: r_to_f64(self.fm),
            fa: r_to_f64(self.fa),
            g: r_to_f64(self.g),
            l: r_to_f64(self.l),
            pm: r_to_f64(self.pm),
            pa: r_to_f64(self.pa),
            osc_type: r_to_f64(self.osc_type),
        }
    }

    pub fn empty() -> Self {
        Self {
            fm: Rational64::new(0, 1),
            fa: Rational64::new(0, 1),
            g: Rational64::new(0, 1),
            l: Rational64::new(0, 1),
            pm: Rational64::new(1, 1),
            pa: Rational64::new(0, 1),
            osc_type: Rational64::new(0, 1),
        }
    }
    pub fn normalize(&mut self, normalizer: &Normalizer) {
        self.fm = normalize_value(self.fm, normalizer.fm.min, normalizer.fm.max);
        self.fa = normalize_value(self.fa, normalizer.fa.min, normalizer.fa.max);
        self.pm = normalize_value(self.pm, normalizer.pm.min, normalizer.pm.max);
        self.pa = normalize_value(self.pa, normalizer.pa.min, normalizer.pa.max);
        self.l = normalize_value(self.l, normalizer.l.min, normalizer.l.max);
        self.g = normalize_value(self.g, normalizer.g.min, normalizer.g.max);
    }

    pub fn denormalize(&mut self, normalizer: &Normalizer) {
        self.fm = denormalize_value(self.fm, normalizer.fm.min, normalizer.fm.max);
        self.fa = denormalize_value(self.fa, normalizer.fa.min, normalizer.fa.max);
        self.pm = denormalize_value(self.pm, normalizer.pm.min, normalizer.pm.max);
        self.pa = denormalize_value(self.pa, normalizer.pa.min, normalizer.pa.max);
        self.l = denormalize_value(self.l, normalizer.l.min, normalizer.l.max);
        self.g = denormalize_value(self.g, normalizer.g.min, normalizer.g.max);
    }

    pub fn new_vec_from_lengths(lengths: Vec<i64>) -> Vec<Self> {
        lengths
            .iter()
            .enumerate()
            .map(|(i, length)| Self {
                fm: Rational64::new(i as i64, 1),
                fa: Rational64::new(0, 1),
                pm: Rational64::new(0, 1),
                pa: Rational64::new(0, 1),
                l: Rational64::new(*length, 1),
                g: Rational64::new(0, 1),
                osc_type: Rational64::new(0, 1),
            })
            .collect()
    }

    pub fn new_vec_from_fm_and_l(fm_and_ls: Vec<(usize, usize)>) -> Vec<Self> {
        fm_and_ls
            .iter()
            .map(|fm_and_l| Self {
                fm: Rational64::new(fm_and_l.0 as i64, 1),
                fa: Rational64::new(0, 1),
                pm: Rational64::new(0, 1),
                pa: Rational64::new(0, 1),
                l: Rational64::new(fm_and_l.1 as i64, 1),
                g: Rational64::new(0, 1),
                osc_type: Rational64::new(0, 1),
            })
            .collect()
    }

    fn from_point_op(op: PointOp) -> Self {
        let osc_type = match op.osc_type {
            OscType::Sine { .. } => Rational64::new(0, 1),
            _ => Rational64::new(1, 1),
        };
        Self {
            fm: op.fm,
            fa: op.fa,
            g: op.g,
            l: op.l,
            pm: op.pm,
            pa: op.pa,
            osc_type,
        }
    }
}

fn make_data_and_expected() -> (VD, VD, VD) {
    let data = vec![
        DataOp::new_vec_from_lengths(vec![1, 3, 3]),
        DataOp::new_vec_from_lengths(vec![1, 1, 1, 2, 1, 1]),
        DataOp::new_vec_from_lengths(vec![2, 2, 2, 1]),
        DataOp::new_vec_from_lengths(vec![7]),
    ];
    let expected1 = vec![
        DataOp::new_vec_from_fm_and_l(vec![(0, 1), (1, 2), (0, 0)]),
        DataOp::new_vec_from_fm_and_l(vec![(0, 1), (1, 1), (2, 1)]),
        DataOp::new_vec_from_fm_and_l(vec![(0, 2), (1, 1), (0, 0)]),
        DataOp::new_vec_from_fm_and_l(vec![(0, 3), (0, 0), (0, 0)]),
    ];

    let expected2 = vec![
        DataOp::new_vec_from_fm_and_l(vec![(1, 3), (2, 1), (0, 0)]),
        DataOp::new_vec_from_fm_and_l(vec![(1, 1), (2, 1), (3, 2)]),
        DataOp::new_vec_from_fm_and_l(vec![(0, 1), (1, 2), (2, 1)]),
        DataOp::new_vec_from_fm_and_l(vec![(0, 4), (0, 0), (0, 0)]),
    ];

    (data, expected1, expected2)
}

#[test]
fn test_datagen() {
    let (data, expected1, expected2) = make_data_and_expected();
    let result = process_normalized(&data, 3);
    assert_eq!(result, expected1);

    let len = shortest_first_element(&data);
    let next = make_next(len, &data);
    let result2 = process_normalized(&next, 3);
    assert_eq!(result2, expected2);
}

pub fn is_not_empty(data: &VD, min_len: usize) -> bool {
    data.iter().any(|voice| voice.len() > min_len)
}

pub fn make_next(l: Rational64, data: &VD) -> VD {
    data.iter()
        .map(|voice| {
            let mut v = voice.to_owned();
            if v[0].l == l {
                v[1..].to_vec()
            } else {
                v[0].l -= l;
                v
            }
        })
        .collect::<VD>()
}

type VD = Vec<Vec<DataOp>>;

pub fn process_normalized(batch: &VD, voice_len: usize) -> VD {
    let batch = take_n(batch, voice_len);
    let min_len = new_find_shortest_phrase(&batch, voice_len);
    let batch = new_make_batch(&batch, min_len, voice_len);
    let batch = pad_voices(&batch, voice_len);

    // let taken = take_n(voice_len, normalized);
    // let (max_idx, min_len) = shortest_phrase(&taken);
    // let batch = make_batch(voice_len, max_idx, min_len, taken);
    // dbg!(max_idx, min_len);
    // unimplemented!();
    batch
}
pub fn take_n(vd: &VD, n: usize) -> VD {
    vd.iter()
        .map(|voice| take_n_voice(voice.to_vec(), n))
        .collect::<VD>()
}

pub fn take_n_voice(voice: Vec<DataOp>, n: usize) -> Vec<DataOp> {
    voice.iter().take(n).map(|op| *op).collect()
}

pub fn pad_voices(vd: &VD, voice_len: usize) -> VD {
    vd.iter()
        .map(|voice| pad_voice(voice.to_vec(), voice_len))
        .collect::<VD>()
}

pub fn new_make_batch(normalized: &VD, max_len: Rational64, n_ops: usize) -> VD {
    let result = normalized
        .iter()
        .map(|voice| new_process_voice(voice.to_vec(), max_len, n_ops))
        .collect::<VD>();
    result
}
pub fn new_process_voice(voice: Vec<DataOp>, max_len: Rational64, n_ops: usize) -> Vec<DataOp> {
    let mut acc = Rational64::new(0, 1);
    let mut idx = 0;
    let mut result = vec![];
    while acc < max_len && idx < n_ops - 1 {
        result.push(voice[idx]);
        idx += 1;
        acc += voice[idx].l;
    }

    if acc > max_len {
        let len = result.len() - 1;
        let diff = result[len].l - (acc - max_len);
        result[len].l = diff;
    }
    result
}

pub fn compress(vd: &VD) -> VD {
    vd.iter()
        .map(|voice| {
            let mut result = vec![voice[0]];
            for op in voice.iter().skip(1) {
                let mut p = result[result.len() - 1];
                if op.fm == p.fm && op.fa == p.fa && op.pa == p.pa && op.pm == p.pm && op.g == p.g {
                    p.l += op.l;
                } else {
                    result.push(*op);
                };
            }
            result
        })
        .collect::<VD>()
}

pub fn new_find_shortest_phrase(vd: &VD, voice_len: usize) -> Rational64 {
    vd.iter()
        .flat_map(|voice| find_length_at_n(voice.to_vec(), voice_len))
        .min()
        .unwrap()
}

pub fn find_length_at_n(dops: Vec<DataOp>, voice_len: usize) -> Option<Rational64> {
    if dops.len() < voice_len {
        return None;
    };
    let mut result = Rational64::new(0, 1);
    for i in 0..voice_len {
        if dops[i].l == Rational64::from_integer(0) {
            return None;
        };
        result += dops[i].l;
    }
    Some(result)
}

// pub fn make_batch(n: usize, max_idx: usize, min_len: Rational64, taken: VD) -> VD {
// let result = taken
// .iter()
// .enumerate()
// .map(|(i, voice)| process_voice(n, i, max_idx, min_len, voice.to_vec()))
// .collect::<VD>();
// result
// }

// fn process_voice(
// n_ops: usize,
// i: usize,
// max_idx: usize,
// min_len: Rational64,
// voice: Vec<DataOp>,
// ) -> Vec<DataOp> {
// if i == max_idx {
// let result = pad_voice(voice, n_ops);
// result
// } else {
// let mut count = Rational64::new(0, 1);
// let mut idx = 0;
// let mut result = vec![];
// while count < min_len && idx < voice.len() {
// result.push(voice[idx]);
// count += voice[idx].l;
// idx += 1
// }

// if count > min_len {
// let len = result.len() - 1;
// let diff = result[len].l - (count - min_len);
// result[len].l = diff;
// }
// // result = pad_voice(n_ops, result);
// result
// }
// }

fn pad_voice(mut voice: Vec<DataOp>, n: usize) -> Vec<DataOp> {
    while voice.len() < n {
        voice.push(DataOp::empty())
    }
    voice
}

// fn take_n(n: usize, normalized: &VD) -> VD {
// normalized
// .iter()
// .map(|voice| {
// voice
// .iter()
// .take(n)
// .map(|op| op.clone())
// .collect::<Vec<DataOp>>()
// })
// .collect::<VD>()
// }

pub fn shortest_phrase(taken: &VD) -> (usize, Rational64) {
    let mut idx = 0;
    let mut min = Rational64::new(i64::MAX, 1);

    for (i, voice) in taken.iter().enumerate() {
        if voice.len() == 3 {
            let mut sum = Rational64::new(0, 1);
            for op in voice {
                sum += op.l
            }
            if sum < min {
                min = sum;
                idx = i;
            }
        }
    }

    (idx, min)
}

pub fn shortest_first_element(data: &VD) -> Rational64 {
    let mut min = Rational64::new(i64::MAX, 1);

    for voice in data.iter() {
        if voice[0].l > Rational64::from_integer(0) && voice[0].l < min {
            min = voice[0].l;
        }
    }

    min
}

fn get_file_names() -> Vec<String> {
    // let demo_dir = "./application/extraResources/demo/";
    let demo_dir = "./nn/new_train/";
    let mut result = vec![];

    let paths = WalkDir::new(demo_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<walkdir::DirEntry>>();

    for entry in paths {
        let f_name = entry.path().to_string_lossy().to_string();
        if f_name.ends_with(".socool")
            && ![
                // "simple.socool",
            ]
            .iter()
            .any(|&name| demo_dir.clone().to_owned() + name == f_name)
        {
            result.push(f_name.to_string());
        }
    }

    result
}

pub fn find_min_max_from_dir() -> Result<(DataOp, DataOp), Error> {
    let mut max_state = DataOp {
        fm: Rational64::new(0, 1),
        fa: Rational64::new(0, 1),
        g: Rational64::new(0, 1),
        l: Rational64::new(0, 1),
        pm: Rational64::new(0, 1),
        pa: Rational64::new(0, 1),
        osc_type: Rational64::new(1, 1),
    };
    let mut min_state = DataOp {
        fm: Rational64::new(0, 1),
        fa: Rational64::new(0, 1),
        g: Rational64::new(0, 1),
        l: Rational64::new(0, 1),
        pm: Rational64::new(0, 1),
        pa: Rational64::new(0, 1),
        osc_type: Rational64::new(1, 1),
    };

    let mut max_ops = 0;
    let mut max_voices = 0;
    let mut n_ops: Vec<usize> = vec![];
    let mut n_voices: Vec<(String, usize)> = vec![];

    for f_name in get_file_names() {
        // println!("{:?}", f_name);
        let render_return = Filename(&f_name).make(RenderType::NfBasisAndTable, None)?;
        let (nf, _, _) = match render_return {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => panic!("huh"),
        };

        max_voices = usize::max(max_voices, nf.operations.len());
        n_voices.push((f_name.to_string(), nf.operations.len()));

        let _data_ops: Vec<Vec<DataOp>> = nf
            .operations
            .iter()
            .map(|voice| {
                max_ops = usize::max(max_ops, voice.len());
                n_ops.push(voice.len());
                // if voice.len() > 3_000 {
                // dbg!("long_voice", &f_name);
                // };
                voice
                    .iter()
                    .map(|op| {
                        let data_op = DataOp::from_point_op(op.to_owned());
                        max_state = DataOp {
                            fm: max_state.fm.max(data_op.fm),
                            fa: max_state.fa.max(data_op.fa),
                            pm: max_state.pm.max(data_op.pm),
                            pa: max_state.pa.max(data_op.pa),
                            g: max_state.g.max(data_op.g),
                            l: max_state.l.max(data_op.l),
                            ..max_state
                        };
                        min_state = DataOp {
                            fm: min_state.fm.min(data_op.fm),
                            fa: min_state.fa.min(data_op.fa),
                            pm: min_state.pm.min(data_op.pm),
                            pa: min_state.pa.min(data_op.pa),
                            g: min_state.g.min(data_op.g),
                            l: min_state.l.min(data_op.l),
                            ..min_state
                        };

                        data_op
                    })
                    .collect()
            })
            .collect();
    }

    println!("MAX {:#?}\nMIN {:#?}\n", max_state, min_state);
    // println!("N VOICES {:#?}\n", n_voices);
    println!("N_Compositions {:#?}\n", n_voices.len());
    println!("Min NOPS {:?}\n", n_ops.iter().min());
    println!("Max NOPS {:?}\n", n_ops.iter().max());
    println!("Max_voices {:?}\n", max_voices);
    Ok((min_state, max_state))
}

pub fn nf_to_normalized_vec_data_op(nf: &NormalForm, normalizer: &Normalizer) -> Vec<Vec<DataOp>> {
    nf.operations
        .iter()
        .map(|voice| {
            voice
                .iter()
                .map(|op| {
                    let mut data_op = DataOp::from_point_op(op.to_owned());
                    data_op.normalize(normalizer);
                    data_op
                })
                .collect()
        })
        .collect()
}
