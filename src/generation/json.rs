use super::{csv::get_length_op4d_1d, op4d::normalize_op4d_1d, Normalizer, TimedOp};
use crate::{
    generation::Op4D,
    ui::{banner, printed},
    write::write_composition_to_json,
};
use num_rational::Rational64;
use scop::Defs;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::path::PathBuf;
use weresocool_ast::{NormalForm, Normalize, Term};
use weresocool_error::Error;
use weresocool_instrument::Basis;

pub fn vec_timed_op_to_vec_op4d(timed_ops: Vec<TimedOp>, basis: &Basis) -> Vec<Op4D> {
    timed_ops.iter().map(|t_op| t_op.to_op_4d(basis)).collect()
}

pub fn composition_to_vec_timed_op(
    composition: &NormalForm,
    defs: &mut Defs<Term>,
) -> Result<(Vec<TimedOp>, usize), Error> {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form, defs)?;

    let n_voices = normal_form.operations.len();
    let mut result: Vec<TimedOp> = normal_form
        .operations
        .iter()
        .enumerate()
        .flat_map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result = vec![];
            let iter = vec_point_op.iter();
            for (event, p_op) in iter.enumerate() {
                let op = TimedOp::from_point_op(p_op, &mut time, voice, event);
                result.push(op);
            }
            result
        })
        .collect();

    result.sort_unstable_by_key(|a| a.t);

    Ok((result, n_voices))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Json1d {
    ops: Vec<Op4D>,
    length: f64,
}

pub fn to_normalized_op4d_1d(
    basis: &Basis,
    composition: &NormalForm,
    defs: &mut Defs<Term>,
    filename: String,
) -> Result<(Vec<Op4D>, f64), Error> {
    banner("JSONIFY-ing".to_string(), filename);

    let (vec_timed_op, _) = composition_to_vec_timed_op(composition, defs)?;
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    //TODO: Factor out
    op4d_1d.retain(|op| {
        let is_silent = op.y == 0.0 || op.z <= 0.0;
        !is_silent
    });

    let max_len = get_length_op4d_1d(&op4d_1d);
    let normalizer = Normalizer::default();
    normalize_op4d_1d(&mut op4d_1d, normalizer);

    Ok((op4d_1d, max_len))
}

pub fn to_json_file(
    basis: &Basis,
    composition: &NormalForm,
    defs: &mut Defs<Term>,
    filename: String,
    output_dir: PathBuf,
) -> Result<(), Error> {
    banner("JSONIFY-ing".to_string(), filename.clone());

    let (op4d_1d, max_len) = to_normalized_op4d_1d(basis, composition, defs, filename.clone())?;

    let json = to_string(&Json1d {
        ops: op4d_1d,
        length: max_len,
    })?;

    write_composition_to_json(&json, &filename, output_dir)?;
    printed("JSON".to_string());

    Ok(())
}
