// use num_rational::Rational64;
use pretty_assertions::assert_eq;
use walkdir::WalkDir;
use weresocool::generation::{RenderReturn, RenderType};
use weresocool::interpretable::{InputType::Filename, Interpretable};
use weresocool_ast::{NormalForm, OscType, PointOp};
use weresocool_error::Error;
use weresocool_shared::helpers::r_to_f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataOp {
    pub fm: f64,
    pub fa: f64,
    pub g: f64,
    pub l: f64,
    pub pm: f64,
    pub pa: f64,
    pub osc_type: f64,
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
    pub fn from_min_max(min: DataOp, max: DataOp) -> Self {
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
    pub min: f64,
    pub max: f64,
}

fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    // equivilance check for floats. max == min.
    let d = if (max - min).abs() < std::f64::EPSILON {
        1.0
    } else {
        max - min
    };
    (value - min) / d
}

impl DataOp {
    pub fn silent() -> Self {
        Self {
            fm: 0.0,
            fa: 0.0,
            g: 0.0,
            l: 0.0,
            pm: 0.0,
            pa: 0.0,
            osc_type: 0.0,
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

    pub fn new_vec_from_lengths(lengths: Vec<usize>) -> Vec<DataOp> {
        lengths
            .iter()
            .enumerate()
            .map(|(i, length)| DataOp {
                fm: i as f64,
                fa: 0.0,
                pm: 0.0,
                pa: 0.0,
                l: *length as f64,
                g: 0.0,
                osc_type: 0.0,
            })
            .collect()
    }

    fn from_point_op(op: PointOp) -> Self {
        let osc_type = match op.osc_type {
            OscType::Sine => 0.0,
            _ => 1.0,
        };
        Self {
            fm: r_to_f64(op.fm),
            fa: r_to_f64(op.fa),
            g: r_to_f64(op.g),
            l: r_to_f64(op.l),
            pm: r_to_f64(op.pm),
            pa: r_to_f64(op.pa),
            osc_type,
        }
    }
}

#[allow(dead_code)]
fn normalize(x: f64, min_x: f64, max_x: f64) -> f64 {
    (x - min_x) / (max_x - min_x)
}

#[test]
fn test_normalize() {
    let result = normalize(8.0, 0.0, 10.0);
    let expected = 0.8;
    assert_eq!(result, expected);
}

fn main() -> Result<(), Error> {
    let (min_state, max_state) = find_min_max_from_dir()?;
    let normalizer = Normalizer::from_min_max(min_state, max_state);

    // let render_return =
    // Filename("songs/tests/mod_by_test.socool").make(RenderType::NfBasisAndTable, None)?;
    // let (nf, _, _) = match render_return {
    // RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
    // _ => panic!("huh"),
    // };

    // let normalized: Vec<Vec<DataOp>> = nf_to_normalized_vec_data_op(&nf, &normalizer);
    let (data, expected) = make_data_and_expected();
    let result = process_normalized(data);
    dbg!(&result);
    assert_eq!(result, expected);

    // let mut file = OpenOptions::new()
    // .create(true)
    // .write(true)
    // .truncate(true)
    // .open("data.csv")
    // .unwrap();

    // for voice in normalized {
    // for op in voice {
    // // dbg!(op);
    // file.write(
    // format!(
    // "{}, {}, {}, {}, {}, {}, {}\n",
    // op.fm.to_string(),
    // op.fa.to_string(),
    // op.pm.to_string(),
    // op.pa.to_string(),
    // op.l.to_string(),
    // op.g.to_string(),
    // op.osc_type.to_string()
    // )
    // .as_bytes(),
    // )?;
    // }
    // }

    Ok(())
}

fn make_data_and_expected() -> (VD, VD) {
    let data = vec![
        DataOp::new_vec_from_lengths(vec![2, 1, 1, 2, 10]),
        DataOp::new_vec_from_lengths(vec![4, 6, 1, 1, 1, 1, 1, 1]),
        DataOp::new_vec_from_lengths(vec![1, 1, 1, 1, 1, 1, 1, 1]),
        DataOp::new_vec_from_lengths(vec![8]),
    ];
    let expected = vec![
        vec![
            DataOp {
                fm: 0.0,
                fa: 0.0,
                g: 0.0,
                l: 2.0,
                pm: 0.0,
                pa: 0.0,
                osc_type: 0.0,
            },
            DataOp {
                fm: 1.0,
                fa: 0.0,
                g: 0.0,
                l: 1.0,
                pm: 0.0,
                pa: 0.0,
                osc_type: 0.0,
            },
        ],
        vec![],
        vec![
            DataOp {
                fm: 0.0,
                fa: 0.0,
                g: 0.0,
                l: 1.0,
                pm: 0.0,
                pa: 0.0,
                osc_type: 0.0,
            },
            DataOp {
                fm: 1.0,
                fa: 0.0,
                g: 0.0,
                l: 1.0,
                pm: 0.0,
                pa: 0.0,
                osc_type: 0.0,
            },
            DataOp {
                fm: 2.0,
                fa: 0.0,
                g: 0.0,
                l: 1.0,
                pm: 0.0,
                pa: 0.0,
                osc_type: 0.0,
            },
        ],
        vec![],
    ];

    (data, expected)
}

type VD = Vec<Vec<DataOp>>;

fn process_normalized(normalized: VD) -> VD {
    let n = 3;
    let taken = take_n(n, &normalized);
    let (max_idx, min_len) = shortest_phrase(&taken);
    let batch = make_batch(n, max_idx, min_len, taken);
    dbg!(max_idx, min_len);
    batch
}

fn make_batch(n: usize, max_idx: usize, min_len: f64, taken: VD) -> VD {
    let result = taken
        .iter()
        .enumerate()
        .map(|(i, voice)| process_voice(n, i, max_idx, min_len, voice.to_vec()))
        .collect::<VD>();
    result
}

// |(i, voice)| match voice.len().partial_cmp(&n).expect("not work") {
// // std::cmp::Ordering::Less => pad_voice(n, voice.to_owned()),
// // _ => vec![],
// },

fn process_voice(
    n_ops: usize,
    i: usize,
    max_idx: usize,
    min_len: f64,
    voice: Vec<DataOp>,
) -> Vec<DataOp> {
    if i == max_idx {
        voice
    } else {
        vec![]
    }
}

fn pad_voice(n: usize, mut voice: Vec<DataOp>) -> Vec<DataOp> {
    for i in 0..voice.len() - n {
        voice.push(DataOp::silent())
    }
    voice
}

fn take_n(n: usize, normalized: &VD) -> VD {
    normalized
        .iter()
        .map(|voice| {
            voice
                .iter()
                .take(n)
                .map(|op| op.clone())
                .collect::<Vec<DataOp>>()
        })
        .collect::<VD>()
}

fn shortest_phrase(taken: &VD) -> (usize, f64) {
    let mut idx = 0;
    let mut min = f64::INFINITY;

    for (i, voice) in taken.iter().enumerate() {
        if voice.len() == 3 {
            let mut sum = 0.0;
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

fn get_file_names() -> Vec<String> {
    let demo_dir = "./application/extraResources/demo/";
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
                "dunno.socool",
                "tokyo.socool",
                "wonder_ball.socool",
                "television.socool",
                "songsongsong.socool",
                "modby.socool",
                "hilbert.socool",
                "how_to_build.socool",
                "how_to_fight.socool",
                "dunno2.socool",
                "for_two_ears.socool",
            ]
            .iter()
            .any(|&name| demo_dir.clone().to_owned() + name == f_name)
        {
            result.push(f_name.to_string());
        }
    }

    result
}

fn find_min_max_from_dir() -> Result<(DataOp, DataOp), Error> {
    let mut max_state = DataOp {
        fm: 0.0,
        fa: 0.0,
        g: 0.0,
        l: 0.0,
        pm: 0.0,
        pa: 0.0,
        osc_type: 1.0,
    };
    let mut min_state = DataOp {
        fm: 0.0,
        fa: 0.0,
        g: 0.0,
        l: 0.0,
        pm: 0.0,
        pa: 0.0,
        osc_type: 0.0,
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

fn nf_to_normalized_vec_data_op(nf: &NormalForm, normalizer: &Normalizer) -> Vec<Vec<DataOp>> {
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
