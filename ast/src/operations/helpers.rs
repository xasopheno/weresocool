use crate::{GetLengthRatio, NormalForm, PointOp, Term, Defs};
use colored::*;
use num_rational::{Ratio, Rational64};
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    collections::VecDeque,
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
    // Use VecDeque for O(1) pop_front instead of O(n) Vec::remove(0)
    let mut m: VecDeque<PointOp> = modulator.iter().cloned().collect();
    let mut i: VecDeque<PointOp> = input.iter().cloned().collect();
    let mut result = Vec::with_capacity(input.len() + modulator.len());

    while let (Some(i_front), Some(m_front)) = (i.front_mut(), m.front()) {
        let mut inpu = i_front.clone();
        let modu = m_front.clone();
        let modu_l = modu.l;
        let inpu_l = inpu.l;

        if modu_l < inpu_l {
            inpu.mod_by(modu, modu_l);
            result.push(inpu);
            i_front.l -= modu_l;
            m.pop_front();
        } else if modu_l > inpu_l {
            inpu.mod_by(modu, inpu_l);
            result.push(inpu);
            // Need to re-borrow m mutably after the immutable borrow ends
            if let Some(m_front_mut) = m.front_mut() {
                m_front_mut.l -= inpu_l;
            }
            i.pop_front();
        } else {
            inpu.mod_by(modu, inpu_l);
            result.push(inpu);
            i.pop_front();
            m.pop_front();
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
    // Use VecDeque for O(1) pop_front instead of O(n) Vec::remove(0)
    let mut m: VecDeque<PointOp> = modulator.iter().cloned().collect();
    let mut i: VecDeque<PointOp> = input.iter().cloned().collect();
    let mut result = Vec::with_capacity(input.len() + modulator.len());

    // Track our position in the overall timeline to know which operation we're in
    let mut position = Ratio::new(0i64, 1i64);

    while let (Some(i_front), Some(m_front)) = (i.front_mut(), m.front()) {
        let mut inpu = i_front.clone();
        let modu = m_front.clone();
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
            i_front.l -= modu_l;
            position += modu_l;
            m.pop_front();
        } else if modu_l > inpu_l {
            inpu.mod_by(modu, inpu_l);
            if is_keeper {
                result.push(inpu);
            }
            // Need to re-borrow m mutably after the immutable borrow ends
            if let Some(m_front_mut) = m.front_mut() {
                m_front_mut.l -= inpu_l;
            }
            position += inpu_l;
            i.pop_front();
        } else {
            inpu.mod_by(modu, inpu_l);
            if is_keeper {
                result.push(inpu);
            }
            position += inpu_l;
            i.pop_front();
            m.pop_front();
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
    result.operations.reserve(l.operations.len());

    for (left, right) in l.operations.iter_mut().zip(r.operations.iter_mut()) {
        left.append(right);
        // Use std::mem::take to move the Vec instead of cloning
        result.operations.push(std::mem::take(left));
    }

    result.length_ratio += r.length_ratio;
    result.length_ratio += l.length_ratio;

    result
}
