use super::{composition_to_vec_timed_op, op4d::normalize_op4d_1d, vec_timed_op_to_vec_op4d, Op4D};
use crate::{ui::banner, write::write_composition_to_csv};
use scop::Defs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use weresocool_ast::{NormalForm, Term};
use weresocool_error::Error;
use weresocool_instrument::Basis;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpCsv1d {
    pub time: f64,
    pub length: f64,
    pub frequency: f64,
    pub pan: f64,
    pub gain: f64,
    pub voice: usize,
    pub event: usize,
}

pub fn to_csv(
    basis: &Basis,
    composition: &NormalForm,
    defs: &mut Defs<Term>,
    filename: String,
    output_dir: PathBuf,
) -> Result<(), Error> {
    banner("CSV-ing".to_string(), filename.clone());

    let (vec_timed_op, _) = composition_to_vec_timed_op(composition, defs)?;
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    op4d_1d.retain(|op| {
        let is_silent = op.y == 0.0 || op.z <= 0.0;
        !is_silent
    });

    normalize_op4d_1d(&mut op4d_1d);

    write_composition_to_csv(&mut op4d_1d, &filename, output_dir)?;

    Ok(())
}

pub fn get_length_op4d_1d(vec_op4d: &[Op4D]) -> f64 {
    let mut max_len: f64 = 0.0;
    for op in vec_op4d {
        max_len = max_len.max(op.t + op.l);
    }

    dbg!(max_len);
    max_len
}
