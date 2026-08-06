use super::{csv::get_length_op4d_1d, op4d::normalize_op4d_1d, TimedOp};
use crate::{
    generation::Op4D,
    ui::{banner, printed},
    write::write_composition_to_json,
};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::path::PathBuf;
use weresocool_ast::{NormalForm, Normalize, Defs};
use weresocool_error::Error;
use weresocool_instrument::Basis;

pub fn vec_timed_op_to_vec_op4d(timed_ops: Vec<TimedOp>, basis: &Basis) -> Vec<Op4D> {
    timed_ops.iter().map(|t_op| t_op.to_op_4d(basis)).collect()
}

pub fn composition_to_vec_timed_op(
    composition: &NormalForm,
    defs: &mut Defs,
) -> Result<(Vec<TimedOp>, usize), Error> {
    let mut normal_form = NormalForm::init();
    println!("Generating Composition\n");
    composition.apply_to_normal_form(&mut normal_form, defs)?;

    let n_voices = normal_form.operations.len();
    
    // Example: 30 FPS => each frame is 1/30 (as a Rational64).
    let frame_length = Rational64::new(1, 30);

    let mut all_timed_ops: Vec<TimedOp> = normal_form
        .operations
        .iter()
        .enumerate()
        .flat_map(|(voice_idx, ops_for_this_voice)| {
            let mut time = Rational64::new(0, 1);
            let mut out = Vec::new();

            // MICROTIMING, resolved here as well as in the render path.
            // There are TWO doors out of a finished NormalForm — this one
            // (JSON / the visual data export) and `nf_to_vec_renderable`
            // (audio, and kintaro's live marks) — and a `Nudge` applied at
            // only one of them would move the sound without moving the
            // picture. Returns `None` when the voice has no nudge, which is
            // every voice in every existing piece.
            let nudged = weresocool_ast::operations::helpers::apply_nudges(
                ops_for_this_voice,
                voice_idx,
            );
            let ops_for_this_voice: &[weresocool_ast::PointOp] =
                nudged.as_deref().unwrap_or(ops_for_this_voice);

            for (event_idx, p_op) in ops_for_this_voice.iter().enumerate() {
                // The total length of the original PointOp
                let mut leftover = p_op.l;

                // Subdivide leftover into increments of frame_length
                while leftover > Rational64::from_integer(0) {
                    // We'll slice off either a full frame or whatever remains
                    let slice_len = if leftover >= frame_length {
                        frame_length
                    } else {
                        leftover
                    };

                    // Clone the original PointOp but override .l to our slice
                    let mut sub_op = p_op.clone();
                    sub_op.l = slice_len;

                    // from_point_op will create the TimedOp at the current `time`
                    // and then increment `time` by `sub_op.l`.
                    let timed_op =
                        TimedOp::from_point_op(&sub_op, &mut time, voice_idx, event_idx);
                    
                    out.push(timed_op);

                    leftover -= slice_len;
                }
            }

            out
        })
        .collect();

    // Sort by start time so the TimedOps are in chronological order:
    all_timed_ops.sort_unstable_by_key(|op| op.t);

    Ok((all_timed_ops, n_voices))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Json1d {
    ops: Vec<Op4D>,
    length: f64,
}

pub fn to_normalized_op4d_1d(
    basis: &Basis,
    composition: &NormalForm,
    defs: &mut Defs,
    filename: String,
) -> Result<(Vec<Op4D>, f64), Error> {
    banner("JSONIFY-ing".to_string(), filename);

    let (vec_timed_op, _) = composition_to_vec_timed_op(composition, defs)?;
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    op4d_1d.retain(|op| {
        let is_silent = op.y == 0.0 || op.z <= 0.0;
        !is_silent
    });

    let max_len = get_length_op4d_1d(&op4d_1d);
    normalize_op4d_1d(&mut op4d_1d);

    Ok((op4d_1d, max_len))
}

pub fn to_json_file(
    basis: &Basis,
    composition: &NormalForm,
    defs: &mut Defs,
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
