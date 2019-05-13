use insta::assert_debug_snapshot_matches;
use num_rational::Rational64;
use socool_ast::ast::{Op, Op::*, OpOrNf::*, OpOrNfTable, OscType};
use socool_parser::parser::*;
use socool_ast::operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use walkdir::WalkDir;
use socool_ast::ast::OpOrNf;
use crate::{
    generation::parsed_to_render::r_to_f64,
    instrument::oscillator::Basis,
};

#[derive(Debug, Clone)]
pub struct DataOp {
    fm: f64,
    fa: f64,
    pm: f64,
    pa: f64,
    g: f64,
    l: f64,
}

#[derive(Debug, Clone)]
pub struct CSVOp {
    pub fm: f64,
    pub fa: f64,
    pub pm: f64,
    pub pa: f64,
    pub g: f64,
    pub t: f64,
}

impl DataOp {
    fn to_csv_op(&self, time: f64) -> CSVOp {
        CSVOp {
            fm: self.fm,
            fa: self.fa,
            pm: self.pm,
            pa: self.pa,
            g: self.g,
            t: time,
        }
    }
}

pub type NormalData = Vec<Vec<DataOp>>;
pub type CSVData = Vec<Vec<CSVOp>>;

pub fn point_op_to_data_op(point_op: &PointOp) -> DataOp {
    let mut result;
    let fm = r_to_f64(point_op.fm);
    if point_op.osc_type == OscType::Sine && fm < 150.0 {
        result = DataOp {
            fm: fm,
            fa: r_to_f64(point_op.fa),
            pm: r_to_f64(point_op.pm),
            pa: r_to_f64(point_op.pa),
            g: r_to_f64(point_op.g),
            l: r_to_f64(point_op.l),
        };
    } else {
        result = DataOp {
            fm: 0.0,
            fa: 0.0,
            pm: 0.0,
            pa: 0.0,
            g: 0.0,
            l: r_to_f64(point_op.l),
        };
    }
    result
}

pub fn composition_to_normal_data(composition: &NormalForm, table: &OpOrNfTable) -> NormalData {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form, table);

    let result: NormalData = normal_form
        .operations
        .iter()
        .map(|vec_point_op| {
            let mut result = vec![];
            for op in vec_point_op {
                let data_op = point_op_to_data_op(op);
                result.push(data_op);
            }
            result
        })
        .collect();

    result
}

pub fn normal_data_to_csv_data(data: NormalData) -> CSVData {
    let subdivision = 0.5;
    let result: CSVData = data
        .iter()
        .map(|vec_data_op| {
            let mut remainder = 0.0;
            let mut seq_time = 0.0;
            let mut result = vec![];
            for op in vec_data_op {
                let op_time = op.l;
                let n_steps_to_push = (op_time + remainder) / subdivision;
                remainder = n_steps_to_push.fract();
                for _n in 0..n_steps_to_push.floor() as usize {
                    let new_op = op.to_csv_op(seq_time);
                    result.push(new_op);
                    seq_time += subdivision;
                }
            }
            result
        })
        .collect();

    result
}

fn get_max_min_csv_data(csv_data: CSVData) -> (CSVOp, CSVOp) {
    let mut max_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        t: 0.0,
    };
    let mut min_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        t: 0.0,
    };

    csv_data.iter().flatten().for_each(|csv_op| {
        max_state = CSVOp {
            fm: max_state.fm.max(csv_op.fm),
            fa: max_state.fa.max(csv_op.fa),
            pm: max_state.pm.max(csv_op.pm),
            pa: max_state.pa.max(csv_op.pa),
            g: max_state.g.max(csv_op.g),
            t: max_state.t.max(csv_op.t),
        };
        min_state = CSVOp {
            fm: min_state.fm.min(csv_op.fm),
            fa: min_state.fa.min(csv_op.fa),
            pm: min_state.pm.min(csv_op.pm),
            pa: min_state.pa.min(csv_op.pa),
            g: min_state.g.min(csv_op.g),
            t: min_state.t.min(csv_op.t),
        };
    });

        (max_state, min_state)
}

pub fn get_min_max_for_path(filename: String) -> (CSVOp, CSVOp){
    let parsed = parse_file(&filename.to_string(), None);
    let parsed_main = parsed.table.get("main").unwrap();

    let nf = match parsed_main {
        OpOrNf::Nf(nf) => nf,
        OpOrNf::Op(_) => panic!("main is Not in Normal Form for some terrible reason."),
    };

    let normal_data = composition_to_normal_data(nf, &parsed.table);
    let csv_data = normal_data_to_csv_data(normal_data);
    let (max, min) = get_max_min_csv_data(csv_data);

    (max, min)
}

#[test]
fn normal_form_to_normal_data_test() {
    let mut normal_form = NormalForm::init();
    let pt = OpOrNfTable::new();

    Overlay {
        operations: vec![
            Op(Sequence {
                operations: vec![
                    Op(PanA {
                        a: Rational64::new(-1, 2),
                    }),
                    Op(TransposeM {
                        m: Rational64::new(2, 1),
                    }),
                    Op(Gain {
                        m: Rational64::new(1, 2),
                    }),
                    Op(Length {
                        m: Rational64::new(2, 1),
                    }),
                ],
            }),
            Op(Sequence {
                operations: vec![Op(Length {
                    m: Rational64::new(5, 1),
                })],
            }),
        ],
    }
    .apply_to_normal_form(&mut normal_form, &pt);

    let normal_data = composition_to_normal_data(&normal_form, &pt);

    assert_debug_snapshot_matches!("normal_form_to_normal_data_test", normal_data);

    let csv_data = normal_data_to_csv_data(normal_data);
    assert_debug_snapshot_matches!("normal_form_to_subdivided_test", csv_data);

    let (max, min) = get_max_min_csv_data(csv_data);
    assert_debug_snapshot_matches!("max_csv_data", max);
    assert_debug_snapshot_matches!("min_csv_data", min);
}

#[test]
fn test_csv_of_file() {
    let mut max_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        t: 0.0,
    };
    let mut min_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        t: 0.0,
    };


    for entry in WalkDir::new("./songs/spring")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let f_name = entry.path().to_string_lossy();

        if f_name.ends_with(".socool") && !f_name.contains("import") {
            let (song_max, song_min) = get_min_max_for_path(f_name.to_string());
            println!("{}", f_name);
            max_state = CSVOp {
                fm: max_state.fm.max(song_max.fm),
                fa: max_state.fa.max(song_max.fa),
                pm: max_state.pm.max(song_max.pm),
                pa: max_state.pa.max(song_max.pa),
                g: max_state.g.max(song_max.g),
                t: max_state.t.max(song_max.t),
            };
            min_state = CSVOp {
                fm: min_state.fm.min(song_min.fm),
                fa: min_state.fa.min(song_min.fa),
                pm: min_state.pm.min(song_min.pm),
                pa: min_state.pa.min(song_min.pa),
                g: min_state.g.min(song_min.g),
                t: min_state.t.min(song_min.t),
            };
        }
    };

    assert_debug_snapshot_matches!("max_for_path", max_state);
    assert_debug_snapshot_matches!("min_for_path", min_state);
}


