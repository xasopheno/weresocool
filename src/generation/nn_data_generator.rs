use num_rational::Rational64;
use socool_ast::ast::{OpOrNf::*, OpOrNfTable, Op::*, Op};
use socool_ast::operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use insta::assert_debug_snapshot_matches;

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

#[derive(Debug, Clone)]
pub struct DataOp {
    fm: f64,
    fa: f64,
    pm: f64,
    pa: f64,
    g: f64,
    t: f64,
}

pub type NormalData = Vec<Vec<DataOp>>;

pub fn point_op_to_data_op(
    point_op: &PointOp,
    time: &mut Rational64
) -> (DataOp, Rational64) {
    let mut new_length = *time + point_op.l;
    let result = DataOp {
        fm: r_to_f64(point_op.fm),
        fa: r_to_f64(point_op.fa),
        pm: r_to_f64(point_op.pm),
        pa: r_to_f64(point_op.pa),
        g: r_to_f64(point_op.g),
        t: r_to_f64(time.clone()),
    };

    (result, new_length)
}

pub fn composition_to_normal_data(composition: &NormalForm, table: &OpOrNfTable) -> NormalData {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form, table);

    let result: NormalData = normal_form
        .operations
        .iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result = vec![];
            for op in vec_point_op {
                let (data_op, new_time) = point_op_to_data_op(op, &mut time);
                result.push(data_op);
                time = new_time;
            }
            result
        })
        .collect();

    result
}

pub fn normal_data_to_csv_data(data: NormalData) -> NormalData {
    let subdivision = 0.5;
//    Should be passed length not time
//    Shouldn't have time yet. :)
    let result: NormalData = data
        .iter()
        .map(|vec_data_op| {
            let mut remainder = 0.0;
            let mut seq_time = 0.0;
            let mut result = vec![];
            for op in vec_data_op {
                let op_time = op.t;
                let n_steps_to_push = (op_time + remainder)/subdivision;
                remainder = n_steps_to_push.fract();
                for n in 0..n_steps_to_push.floor() as usize {
                    let mut new_op = op.clone();
                    new_op.t = seq_time;
                    result.push(new_op);
                    seq_time += subdivision;
                }
            }
            result
        })
        .collect();

    result
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
                         a: Rational64::new(1, 2),
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

    let normal_data_subdivided = normal_data_to_csv_data(normal_data);
    assert_debug_snapshot_matches!("normal_form_to_subdivided_test", normal_data_subdivided);
}
