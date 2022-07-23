use super::{
    composition_to_vec_timed_op, op4d::normalize_op4d_1d, vec_timed_op_to_vec_op4d, MinMax,
    Normalizer, Op4D,
};
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

    let (normalizer, _max_len) = get_min_max_op4d_1d(&op4d_1d);

    normalize_op4d_1d(&mut op4d_1d, normalizer);

    write_composition_to_csv(&mut op4d_1d, &filename, output_dir)?;

    Ok(())
}

pub fn get_min_max_op4d_1d(vec_op4d: &[Op4D]) -> (Normalizer, f64) {
    let mut max_state = Op4D {
        t: 0.0,
        event: 0,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
        names: vec![],
    };

    let mut min_state = Op4D {
        t: 0.0,
        event: 10,
        voice: 10,
        x: 0.0,
        y: 10_000.0,
        z: 1.0,
        l: 1.0,
        names: vec![],
    };

    let mut max_len: f64 = 0.0;
    for op in vec_op4d {
        max_len = max_len.max(op.t + op.l);

        max_state = Op4D {
            x: max_state.x.max((op.x).abs()),
            y: max_state.y.max(op.y),
            z: max_state.z.max(op.z),
            l: max_state.l.max(op.l),
            t: max_state.t.max(op.t),
            event: max_state.event.max(op.event),
            voice: max_state.voice.max(op.voice),
            names: vec![],
        };

        min_state = Op4D {
            x: min_state.x.min(-(op.x).abs()),
            y: min_state.y.min(op.y),
            z: min_state.z.min(op.z),
            l: min_state.l.min(op.l),
            t: min_state.t.min(op.t),
            event: min_state.event.min(op.event),
            voice: min_state.voice.min(op.voice),
            names: vec![],
        };
    }

    let n = Normalizer {
        x: MinMax {
            min: min_state.x,
            max: max_state.x,
        },
        y: MinMax {
            min: min_state.y,
            max: max_state.y,
        },
        z: MinMax {
            min: min_state.z,
            max: max_state.z,
        },
    };
    dbg!(n);
    dbg!(max_len);
    (n, max_len)
}
