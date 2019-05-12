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
    mut time: &mut Rational64
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

    let mut result: NormalData = normal_form
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

    let result = composition_to_normal_data(&normal_form, &pt);

    assert_debug_snapshot_matches!("normal_form_to_normal_data_test", result);
}
