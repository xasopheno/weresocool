use difference::{Changeset, Difference};
use indexmap::IndexMap;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_string_pretty};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use weresocool::{
    generation::parsed_to_render::{generate_waveforms, sum_all_waveforms},
    instrument::{Basis, Normalize, StereoWaveform},
    renderable::nf_to_vec_renderable,
};
use weresocool_ast::{NormalForm, Normalize as NormalizeOp};
use weresocool_parser::*;
pub mod expect;
//mod tests;

type TestTable = IndexMap<String, CompositionHashes>;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionHashes {
    op: u64,
    normal_form: u64,
    stereo_waveform: f64,
    pop_check: bool,
}

pub fn read_test_table_from_json_file() -> TestTable {
    let file = File::open("src/snapshot/hashes.json").unwrap();

    let mut decoded: TestTable = from_reader(&file).unwrap();
    decoded.sort_by(|a, _b, c, _d| a.partial_cmp(c).unwrap());
    decoded
}

pub fn generate_test_table() -> TestTable {
    let mut test_table: TestTable = IndexMap::new();
    let paths = fs::read_dir("../songs/test/").unwrap();
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
    let mut file = File::create("src/snapshot/hashes.json").unwrap();
    file.write_all(pretty.as_bytes()).unwrap();
}

fn generate_render_hashes(p: &str) -> CompositionHashes {
    let vec_string = filename_to_vec_string(&p.to_string());
    let parsed = parse_file(vec_string, None).unwrap();
    let main_op = parsed.defs.terms.get("main").unwrap();
    let init = parsed.init;
    let op_hash = calculate_hash(main_op);
    let mut normal_form = NormalForm::init();

    main_op.apply_to_normal_form(&mut normal_form, &parsed.defs);
    let nf_hash = calculate_hash(&normal_form);

    let origin = Basis {
        f: init.f,
        g: init.g,
        l: init.l,
        p: init.p,
        a: Rational64::new(1, 1),
        d: Rational64::new(1, 1),
    };

    let renderable = nf_to_vec_renderable(&normal_form, &parsed.defs, &origin);

    let vec_wav = generate_waveforms(renderable, false);
    let mut result = sum_all_waveforms(vec_wav);
    let pop_check = pop_check(&result);

    result.normalize();

    let render_hash = sum_vec(result.l_buffer) + sum_vec(result.r_buffer);
    let render_hash = (render_hash * 10_000_000_000_000.0).ceil() / 10_000_000_000_000.0;
    let render_hash_string = &render_hash.to_string()[..12];

    CompositionHashes {
        op: op_hash,
        normal_form: nf_hash,
        stereo_waveform: render_hash_string.parse().unwrap(),
        pop_check,
    }
}

fn pop_check(stereo_waveform: &StereoWaveform) -> bool {
    let mut max_d = 0.0;

    for i in 1..stereo_waveform.r_buffer.len() {
        let d = stereo_waveform.r_buffer[i] - stereo_waveform.r_buffer[i - 1];
        if d.abs() > max_d {
            max_d = d.abs();
        }
    }

    for i in 1..stereo_waveform.l_buffer.len() {
        let d = stereo_waveform.l_buffer[i] - stereo_waveform.l_buffer[i - 1];
        if d.abs() > max_d {
            max_d = d.abs();
        }
    }

    max_d < 0.20
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
pub fn show_difference(tt1: TestTable, tt2: TestTable) {
    let Changeset { diffs, .. } = Changeset::new(
        &to_string_pretty(&tt1).unwrap(),
        &to_string_pretty(&tt2).unwrap(),
        "\n",
    );

    let mut terminal = term::stdout().unwrap();

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                terminal.reset().unwrap();
                writeln!(terminal, " {}", x);
            }
            Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        terminal.fg(term::color::GREEN).unwrap();
                        write!(terminal, "+");
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    terminal.fg(term::color::GREEN).unwrap();
                                    write!(terminal, "{}", z);
                                    write!(terminal, " ");
                                }
                                Difference::Add(ref z) => {
                                    terminal.fg(term::color::WHITE).unwrap();
                                    terminal.bg(term::color::GREEN).unwrap();
                                    write!(terminal, "{}", z);
                                    terminal.reset().unwrap();
                                    write!(terminal, " ");
                                }
                                _ => (),
                            }
                        }
                        writeln!(terminal);
                    }
                    _ => {
                        terminal.fg(term::color::BRIGHT_GREEN).unwrap();
                        writeln!(terminal, "+{}", x);
                    }
                };
            }
            Difference::Rem(ref x) => {
                terminal.fg(term::color::RED).unwrap();
                writeln!(terminal, "-{}", x);
            }
        }
    }
    terminal.reset().unwrap();
    terminal.flush().unwrap();
}
