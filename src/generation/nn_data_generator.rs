use socool_ast::operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use socool_ast::ast::OpOrNfTable;

pub struct DataOp {
    f: f64,
    g: f64,
    p: f64,
}

pub type NormalData = Vec<Vec<DataOp>>;


pub fn composition_to_normal_data(composition: &NormalForm, table: &OpOrNfTable) -> NormalData {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form, table);

    let mut result: NormalData = normal_form
        .operations
        .iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
//            let mut time = Rational64::new(0, 1);
//            let mut result = vec![];
//            vec_point_op.iter().enumerate().for_each(|(event, p_op)| {
//                let (on, off) = point_op_to_timed_op(p_op, &mut time, voice, event);
//                result.push(on);
//                result.push(off);
//            });
//            result
        })
        .collect();

    result.sort_unstable_by_key(|a| a.t);

    result
}
