use crate::generation::{normalform_to_vec_timed_op_1d, TimedOp};
use insta::assert_debug_snapshot_matches;
use num_rational::Rational64;
use socool_ast::OpOrNf;
use socool_parser::parser::*;

#[derive(Debug, Clone)]
pub struct CSVOp {
    pub fm: f64,
    pub fa: f64,
    pub pm: f64,
    pub pa: f64,
    pub g: f64,
    pub l: f64,
    pub v: usize,
}

#[derive(Debug, Clone)]
pub struct Normalizer {
    pub fm: (Rational64, Rational64),
    pub fa: (Rational64, Rational64),
    pub pm: (Rational64, Rational64),
    pub pa: (Rational64, Rational64),
    pub g: (Rational64, Rational64),
    pub l: (Rational64, Rational64),
    pub v: (Rational64, Rational64),
}

pub type CSVData = Vec<CSVOp>;

pub fn vec_timed_op_to_vec_csv_data(timed_ops: Vec<TimedOp>) -> CSVData {
    timed_ops.iter().map(|t_op| t_op.to_csv_op()).collect()
}

pub fn vec_timed_op_1d_to_csv_data_1d(timed_ops: Vec<TimedOp>) -> Vec<CSVOp> {
    timed_ops.iter().map(|t_op| t_op.to_csv_op()).collect()
}

pub fn vec_timed_op_2d_to_csv_data_2d(timed_ops: Vec<Vec<TimedOp>>) -> Vec<Vec<CSVOp>> {
    let result: Vec<Vec<CSVOp>> = timed_ops
        .iter()
        .map(|vec_t_op| vec_t_op.iter().map(|t_op| t_op.to_csv_op()).collect())
        .collect();
    result
}

//pub fn csv_data_to_normalized_csv_data(data: CSVData, ) {
//    let max__state = CSVOp {
//        fm: 70.0,
//        fa: 33.0,
//        pm: 1.0,
//        pa: 1.7857142857142858,
//        g: 1.0,
//        l: 576.0,
//        v: 191
//    }
//
//}

fn get_max_min_csv_data(csv_data: CSVData) -> (CSVOp, CSVOp) {
    let mut max_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        l: 0.0,
        v: 0,
    };
    let mut min_state = CSVOp {
        fm: 0.0,
        fa: 0.0,
        pm: 0.0,
        pa: 0.0,
        g: 0.0,
        l: 0.0,
        v: 0,
    };

    csv_data.iter().for_each(|csv_op| {
        max_state = CSVOp {
            fm: max_state.fm.max(csv_op.fm),
            fa: max_state.fa.max(csv_op.fa),
            pm: max_state.pm.max(csv_op.pm),
            pa: max_state.pa.max(csv_op.pa),
            g: max_state.g.max(csv_op.g),
            l: max_state.l.max(csv_op.l),
            v: max_state.v.max(csv_op.v),
        };
        min_state = CSVOp {
            fm: min_state.fm.min(csv_op.fm),
            fa: min_state.fa.min(csv_op.fa),
            pm: min_state.pm.min(csv_op.pm),
            pa: min_state.pa.min(csv_op.pa),
            g: min_state.g.min(csv_op.g),
            l: min_state.l.min(csv_op.l),
            v: min_state.v.min(csv_op.v),
        };
    });

    (max_state, min_state)
}

pub fn get_min_max_for_path(filename: String) -> (CSVOp, CSVOp, usize) {
    let parsed = parse_file(&filename.to_string(), None);
    let parsed_main = parsed.table.get("main").unwrap();

    let nf = match parsed_main {
        OpOrNf::Nf(nf) => nf,
        OpOrNf::Op(_) => panic!("main is Not in Normal Form for some terrible reason."),
    };

    let timed_ops = normalform_to_vec_timed_op_1d(nf, &parsed.table);
    let n_voices = timed_ops.len();
    let csv_data = vec_timed_op_1d_to_csv_data_1d(timed_ops);
    dbg!(csv_data.clone());
    let (max, min) = get_max_min_csv_data(csv_data);

    (max, min, n_voices)
}

#[cfg(test)]
mod nn_data_test {
    use super::*;
    use socool_ast::{NormalForm, Normalize, Op::*, OpOrNf::*, OpOrNfTable, PointOp};
    use walkdir::WalkDir;

    //    #[test]
    //    fn normal_form_to_normal_data_test() {
    //        let mut normal_form = NormalForm::init();
    //        let pt = OpOrNfTable::new();
    //
    //        Overlay {
    //            operations: vec![
    //                Op(Sequence {
    //                    operations: vec![
    //                        Op(PanA {
    //                            a: Rational64::new(-1, 2),
    //                        }),
    //                        Op(TransposeM {
    //                            m: Rational64::new(2, 1),
    //                        }),
    //                        Op(Gain {
    //                            m: Rational64::new(1, 2),
    //                        }),
    //                        Op(Length {
    //                            m: Rational64::new(2, 1),
    //                        }),
    //                    ],
    //                }),
    //                Op(Sequence {
    //                    operations: vec![Op(Length {
    //                        m: Rational64::new(5, 1),
    //                    })],
    //                }),
    //            ],
    //        }
    //        .apply_to_normal_form(&mut normal_form, &pt);
    //
    //        let normal_data = normalform_to_vec_timed_op_1d(&normal_form, &pt);
    //
    //        assert_debug_snapshot_matches!("normal_form_to_normal_data_test", normal_data);
    //
    //        let csv_data = vec_timed_op_to_vec_csv_data(normal_data);
    //        assert_debug_snapshot_matches!("normal_form_to_subdivided_test", csv_data);
    //
    //        let (max, min) = get_max_min_csv_data(csv_data);
    //        assert_debug_snapshot_matches!("max_csv_data", max);
    //        assert_debug_snapshot_matches!("min_csv_data", min);
    //    }

    #[test]
    fn test_csv_of_file() {
        let mut max_state = CSVOp {
            fm: 0.0,
            fa: 0.0,
            pm: 0.0,
            pa: 0.0,
            g: 0.0,
            l: 0.0,
            v: 0,
        };
        let mut min_state = CSVOp {
            fm: 0.0,
            fa: 0.0,
            pm: 0.0,
            pa: 0.0,
            g: 0.0,
            l: 0.0,
            v: 0,
        };

        for entry in WalkDir::new("./songs/test_data/")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let f_name = entry.path().to_string_lossy();

            if f_name.ends_with(".socool") && !f_name.contains("import") {
                let (song_max, song_min, n_events) = get_min_max_for_path(f_name.to_string());
                println!("{}", f_name);
                max_state = CSVOp {
                    fm: max_state.fm.max(song_max.fm),
                    fa: max_state.fa.max(song_max.fa),
                    pm: max_state.pm.max(song_max.pm),
                    pa: max_state.pa.max(song_max.pa),
                    g: max_state.g.max(song_max.g),
                    l: max_state.l.max(song_max.l),
                    v: max_state.v.max(song_max.v),
                };
                min_state = CSVOp {
                    fm: min_state.fm.min(song_min.fm),
                    fa: min_state.fa.min(song_min.fa),
                    pm: min_state.pm.min(song_min.pm),
                    pa: min_state.pa.min(song_min.pa),
                    g: min_state.g.min(song_min.g),
                    l: min_state.l.min(song_min.l),
                    v: min_state.v.min(song_min.v),
                };
            }
        }

        assert_debug_snapshot_matches!("test_csv_of_file_max_for_path", max_state);
        assert_debug_snapshot_matches!("test_csv_of_file_min_for_path", min_state);
    }

}
