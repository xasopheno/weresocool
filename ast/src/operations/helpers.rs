use crate::{GetLengthRatio, NormalForm, Normalize, PointOp, Term, Defs};
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
            let name: String = id.into();
            let known = defs.ops.visible_names();
            let did_you_mean = weresocool_error::nearest_names(&name, known.iter());
            println!("Not able to find {} in let defs", name.red().bold());
            Err(IdError { id: name, did_you_mean }.into_error())
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

/// ZIP — element-wise combination of op sequences (isorhythm).
///
/// `Zip [A, B, C]` pairs A's events with B's and C's by position and
/// multiplies them together, cycling the later operands against the first.
/// The classical name for the two-operand case is isorhythm: a *color* (the
/// pitch series) running against a *talea* (the rhythm series) of a
/// different length, so the accents land somewhere new each time round.
///
/// ```text
/// Zip [ Seq [Fm 1, Fm 2, Fm 3, Fm 4, Fm 5, Fm 6, Fm 7],
///       Seq [Lm 3, Lm 2] | Lm 1/5 ]
/// ```
///
/// WHY THIS WORKS AT ALL: unused fields are identity. A rhythm written with
/// only `Lm` normalizes to points with `fm = 1, g = 1, pm = 1`, so
/// multiplying it into the melody contributes ONLY length. No field
/// selectors and no masks are needed — the identity elements do the routing.
/// The flip side, worth knowing: an operand that carries incidental gain or
/// pan WILL impose it. That is a feature (a dynamic contour is just another
/// operand) but it surprises the first time.
///
/// TWO RULES, AND THEY ARE THE SAME RULE:
///
/// 1. The FIRST operand is the subject. It receives whatever was piped in;
///    the rest normalize on their own against a unit form, because they are
///    patterns rather than subjects. So `Zip [x]` is exactly `x`.
/// 2. The first operand governs LENGTH. Later operands cycle under it and
///    never extend it.
///
/// Rule 2 is why there is no lcm here. Running 7 against 2 to their
/// realignment at 14 is the musically interesting case, but making that the
/// default hides the piece's duration (11 against 13 silently becomes 143
/// events) and you cannot read the length off the page. Phasing is opt-in
/// with an op that already exists:
///
/// ```text
/// Zip [ Seq [...7 pitches] | Repeat 2, Seq [Lm 3, Lm 2] | Lm 1/5 ]
/// ```
pub fn zip_terms(
    operations: &[Term],
    input: &NormalForm,
    defs: &mut Defs,
) -> Result<NormalForm, Error> {
    let Some((subject_term, pattern_terms)) = operations.split_first() else {
        return Ok(input.clone());
    };

    let mut subject = input.clone();
    subject_term.apply_to_normal_form(&mut subject, defs)?;

    // Patterns normalize against a UNIT form, not against `input`. Zipping
    // them against `input` too would multiply the input's own fm and length
    // into the result once per operand.
    let mut patterns: Vec<NormalForm> = Vec::with_capacity(pattern_terms.len());
    for term in pattern_terms {
        let mut nf = NormalForm::init();
        term.apply_to_normal_form(&mut nf, defs)?;
        patterns.push(nf);
    }

    for (v, voice) in subject.operations.iter_mut().enumerate() {
        for (i, point) in voice.iter_mut().enumerate() {
            for pattern in &patterns {
                if pattern.operations.is_empty() {
                    continue;
                }
                let pattern_voice = &pattern.operations[v % pattern.operations.len()];
                if pattern_voice.is_empty() {
                    continue;
                }
                // PointOp's own `Mul`: fm/pm/g/l multiply, fa/pa add, names
                // union. Non-multiplicative fields (osc_type, asr, is_out)
                // take the RIGHT operand, so a later operand in the Zip wins
                // for those — same precedence as writing it later in a pipe.
                *point = point.clone() * pattern_voice[i % pattern_voice.len()].clone();
            }
        }
    }

    // Voices can come out of the zip at different total lengths: the subject's
    // voices need not hold the same number of points, and each one cycles the
    // patterns on its own index. A NormalForm's voices are parallel, so the
    // short ones get padded with silence rather than left ragged.
    let totals: Vec<Rational64> = subject
        .operations
        .iter()
        .map(|voice| {
            voice
                .iter()
                .fold(Ratio::new(0, 1), |acc, point| acc + point.l)
        })
        .collect();
    let max = totals
        .iter()
        .copied()
        .max()
        .unwrap_or_else(|| Ratio::new(0, 1));
    for (voice, total) in subject.operations.iter_mut().zip(totals) {
        if total < max {
            voice.push(PointOp {
                fm: Ratio::new(0, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: max - total,
                ..Default::default()
            });
        }
    }
    subject.length_ratio = max;

    Ok(subject)
}
