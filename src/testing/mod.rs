use crate::generation::parsed_to_render::{generate_waveforms, sum_all_waveforms};
use crate::generation::renderable::nf_to_vec_renderable;
use crate::instrument::{Basis, Normalize};
use difference::{Changeset, Difference};
use indexmap::IndexMap;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_string_pretty};
use socool_ast::{NormalForm, Normalize as NormalizeOp};
use socool_parser::*;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;

type TestTable = IndexMap<String, CompositionHashes>;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionHashes {
    op: u64,
    normal_form: u64,
    stereo_waveform: f64,
}

pub fn read_test_table_from_json_file() -> TestTable {
    //    let file = File::open("src/testing/hashes.json").unwrap();
    let file = File::open("src/testing/hashes.json").unwrap();

    let mut decoded: TestTable = from_reader(&file).unwrap();
    decoded.sort_by(|a, _b, c, _d| a.partial_cmp(c).unwrap());
    decoded
}

pub fn generate_test_table() -> TestTable {
    let mut test_table: TestTable = IndexMap::new();
    //    let paths = fs::read_dir("./songs/test").unwrap();
    let paths = fs::read_dir("./songs/test").unwrap();
    for path in paths {
        let p = path.unwrap().path().into_os_string().into_string().unwrap();
        if p.ends_with(".socool") {
            println!("{}", p.clone());
            let composition_hashes = generate_render_hashes(&p);
            test_table.insert(p, composition_hashes);
        }
    }

    test_table.sort_by(|a, _b, c, _d| a.partial_cmp(c).unwrap());
    test_table
}

pub fn write_test_table_to_json_file(test_table: &TestTable) {
    let pretty = to_string_pretty(test_table).unwrap();
    //    let mut file = File::create("src/testing/hashes.json").unwrap();
    let mut file = File::create("src/testing/hashes.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn generate_render_hashes(p: &String) -> CompositionHashes {
    let parsed = parse_file(p, None);
    let main_op = parsed.table.get("main").unwrap();
    let init = parsed.init;
    let op_hash = calculate_hash(main_op);
    let mut normal_form = NormalForm::init();

    main_op.apply_to_normal_form(&mut normal_form, &parsed.table);
    let nf_hash = calculate_hash(&normal_form);

    let origin = Basis {
        f: init.f,
        g: init.g,
        l: init.l,
        p: init.p,
        a: Rational64::new(1, 1),
        d: Rational64::new(1, 1),
    };

    let renderable = nf_to_vec_renderable(&normal_form, &parsed.table, &origin);

    let vec_wav = generate_waveforms(renderable, false);
    let mut result = sum_all_waveforms(vec_wav);

    result.normalize();

    let render_hash = sum_vec(result.l_buffer) + sum_vec(result.r_buffer);
    let render_hash = (render_hash * 1_000_000_000_000.0).ceil() / 1_000_000_000_000.0;
    let render_hash_string = &render_hash.to_string()[..13];

    let hashes = CompositionHashes {
        op: op_hash,
        normal_form: nf_hash,
        stereo_waveform: render_hash_string.parse().unwrap(),
    };

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

#[allow(unused_must_use)]
pub fn show_difference(a: TestTable, b: TestTable) {
    let Changeset { diffs, .. } = Changeset::new(
        &to_string_pretty(&a).unwrap(),
        &to_string_pretty(&b).unwrap(),
        "\n",
    );

    let mut t = term::stdout().unwrap();

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                t.reset().unwrap();
                writeln!(t, " {}", x);
            }
            Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        t.fg(term::color::GREEN).unwrap();
                        write!(t, "+");
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    t.fg(term::color::GREEN).unwrap();
                                    write!(t, "{}", z);
                                    write!(t, " ");
                                }
                                Difference::Add(ref z) => {
                                    t.fg(term::color::WHITE).unwrap();
                                    t.bg(term::color::GREEN).unwrap();
                                    write!(t, "{}", z);
                                    t.reset().unwrap();
                                    write!(t, " ");
                                }
                                _ => (),
                            }
                        }
                        writeln!(t, "");
                    }
                    _ => {
                        t.fg(term::color::BRIGHT_GREEN).unwrap();
                        writeln!(t, "+{}", x);
                    }
                };
            }
            Difference::Rem(ref x) => {
                t.fg(term::color::RED).unwrap();
                writeln!(t, "-{}", x);
            }
        }
    }
    t.reset().unwrap();
    t.flush().unwrap();
}
