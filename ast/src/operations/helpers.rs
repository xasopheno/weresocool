use crate::{GetLengthRatio, NormalForm, PointOp, Term, Defs};
use colored::*;
use num_rational::{Ratio, Rational64};
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    fmt::Display,
};
use weresocool_error::{Error, IdError};

pub fn handle_id_error<S: Into<String> + Clone + Display + std::fmt::Debug>(
    id: S,
    defs: &Defs,
) -> Result<Term, Error> {
    match defs.ops.get(&id.clone().into()) {
        Some(result) => Ok(result.to_owned()),
        None => {
            println!(
                "Not able to find {} in let defs",
                id.to_string().red().bold()
            );
            Err(IdError { id: id.into() }.into_error())
        }
    }
}

pub fn modulate(input: &[PointOp], modulator: &[PointOp]) -> Vec<PointOp> {
    let mut m = modulator.to_owned();
    let mut i = input.to_owned();
    let mut result = vec![];
    while !m.is_empty() && !i.is_empty() {
        let mut inpu = i[0].clone();
        let modu = m[0].clone();
        let modu_l = modu.l;
        let inpu_l = inpu.l;
        if modu_l < inpu_l {
            inpu.mod_by(modu, modu_l);
            result.push(inpu);

            i[0].l -= modu_l;

            m.remove(0);
        } else if modu.l > inpu.l {
            inpu.mod_by(modu, inpu_l);
            result.push(inpu);

            m[0].l -= inpu_l;

            i.remove(0);
        } else {
            inpu.mod_by(modu, inpu_l);
            result.push(inpu);

            i.remove(0);
            m.remove(0);
        }
    }

    result
}

/// Like modulate, but only keeps pieces that fall within "keeper" regions
/// op_lengths: the length of each operation in the slice
/// keeper_indices: which operation indices should be kept (not spacers)
pub fn slice_modulate(
    input: &[PointOp],
    modulator: &[PointOp],
    op_lengths: &[Rational64],
    keeper_indices: &[usize],
) -> Vec<PointOp> {
    let mut m = modulator.to_owned();
    let mut i = input.to_owned();
    let mut result = vec![];

    // Track our position in the overall timeline to know which operation we're in
    let mut position = Ratio::new(0i64, 1i64);

    while !m.is_empty() && !i.is_empty() {
        let mut inpu = i[0].clone();
        let modu = m[0].clone();
        let modu_l = modu.l;
        let inpu_l = inpu.l;

        // Determine which operation index this piece belongs to
        let op_idx = get_operation_index(position, op_lengths);
        let is_keeper = keeper_indices.contains(&op_idx);

        if modu_l < inpu_l {
            inpu.mod_by(modu, modu_l);
            if is_keeper {
                result.push(inpu);
            }

            i[0].l -= modu_l;
            position += modu_l;

            m.remove(0);
        } else if modu.l > inpu.l {
            inpu.mod_by(modu, inpu_l);
            if is_keeper {
                result.push(inpu);
            }

            m[0].l -= inpu_l;
            position += inpu_l;

            i.remove(0);
        } else {
            inpu.mod_by(modu, inpu_l);
            if is_keeper {
                result.push(inpu);
            }

            position += inpu_l;

            i.remove(0);
            m.remove(0);
        }
    }

    result
}

/// Given a position in the timeline and operation lengths, return which operation index we're in
fn get_operation_index(position: Rational64, op_lengths: &[Rational64]) -> usize {
    let mut cumulative = Ratio::new(0i64, 1i64);
    for (idx, &len) in op_lengths.iter().enumerate() {
        cumulative += len;
        if position < cumulative {
            return idx;
        }
    }
    // If we're at the very end, return the last index
    op_lengths.len().saturating_sub(1)
}

pub fn pad_length(
    input: &mut NormalForm,
    max_len: Rational64,
    defs: &mut Defs,
) -> Result<(), Error> {
    let input_lr = input.get_length_ratio(input, defs)?;
    if max_len > Rational64::new(0, 1) && input_lr < max_len {
        for voice in input.operations.iter_mut() {
            voice.push(PointOp {
                fm: Ratio::new(0, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: max_len - input_lr,
                ..Default::default()
            });
        }
    }
    input.length_ratio = max_len;
    Ok(())
}

pub fn join_sequence(mut l: NormalForm, mut r: NormalForm) -> NormalForm {
    if l.operations.is_empty() {
        return r;
    }

    let diff = l.operations.len() as isize - r.operations.len() as isize;
    match diff.partial_cmp(&0).unwrap() {
        Equal => {}
        Greater => {
            for _ in 0..diff {
                r.operations.push(vec![PointOp {
                    fm: Ratio::new(0, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(0, 1),
                    l: r.length_ratio,
                    ..Default::default()
                }])
            }
        }
        Less => {
            for _ in 0..-diff {
                l.operations.push(vec![PointOp {
                    fm: Ratio::new(0, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(0, 1),
                    l: l.length_ratio,
                    ..Default::default()
                }])
            }
        }
    }

    let mut result = NormalForm::init_empty();

    for (left, right) in l.operations.iter_mut().zip(r.operations.iter_mut()) {
        left.append(right);

        result.operations.push(left.clone());
    }

    result.length_ratio += r.length_ratio;
    result.length_ratio += l.length_ratio;

    result
}
