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

/// ZIP — run a rhythm under a melody (isorhythm).
///
/// `x | Zip [Lm 3, Lm 1]` walks x's events and applies the bracket list to
/// them in turn — long, short, long, short — cycling the list when it runs
/// short. THE BRACKET LIST IS THE RHYTHM, one step per element:
///
/// ```text
/// Seq [Fm 1, Fm 2, Fm 3, Fm 4, Fm 5, Fm 6, Fm 7]
///   | Zip [Lm 3, Lm 2]
/// ```
///
/// Seven pitches under a two-step talea. The classical name is isorhythm: a
/// *color* (the pitch series) running against a *talea* (the rhythm series)
/// of a different length, so the accents land somewhere new each time round.
/// Swing is the two-step case, `Zip [Lm 4/3, Lm 2/3]` — the pair sums to 2,
/// same as two straight notes, so the phrase length does not move and only
/// the placement does.
///
/// THE INVARIANT, and the reason the subject comes through the pipe rather
/// than as a first argument:
///
/// > **Zip never adds or removes events. It multiplies fields into the
/// > events already there.**
///
/// That makes Zip a MODIFIER — it belongs with `Lm`, `Gm` and `Reverse`, not
/// with `Seq` and `Overlay`, which build events. Writing the subject inside
/// the bracket list would dress it as a constructor and force a "the first
/// one is special" rule that cannot be read off the page.
///
/// WHY IT WORKS AT ALL: unused fields are identity. A rhythm written with
/// only `Lm` normalizes to points with `fm = 1, g = 1, pm = 1`, so applying
/// it to a melody contributes ONLY length. No field selectors and no masks
/// are needed — the identity elements do the routing. The flip side, worth
/// knowing: a step that carries incidental gain or pan WILL impose it. That
/// is a feature (a dynamic contour is just another cycle) but it surprises
/// the first time.
///
/// Want several cycles at once? Chain them. Each `Zip` is one talea:
///
/// ```text
/// tune | Zip [Lm 4/3, Lm 2/3] | Zip [Gm 1, Gm 1/2, Gm 3/4]
/// ```
///
/// The talea CYCLES and never extends the subject, so the piece's length is
/// the subject's length and you can read it off the page. Running 7 against
/// 2 to their realignment at 14 is the musically interesting case, but as a
/// default it would hide the duration — 11 against 13 would silently become
/// 143 events. So it is opt-in, with an op that already exists:
///
/// ```text
/// tune | Repeat 2 | Zip [Lm 3, Lm 2]
/// ```
pub fn zip_terms(
    operations: &[Term],
    input: &NormalForm,
    defs: &mut Defs,
) -> Result<NormalForm, Error> {
    let mut subject = input.clone();

    // THE BRACKET LIST IS THE RHYTHM. `Zip [Lm 3, Lm 1]` is one talea of two
    // steps — long, short, long, short — not two separate patterns.
    //
    // It read the other way first, as a list of patterns applied at once, and
    // that multiplied them: `Zip [Lm 3, Lm 1]` gave every note l * 3 * 1, a
    // uniform stretch with no alternation whatsoever. It rendered, it sounded
    // nearly right, and nothing said the rhythm had not been applied. Wanting
    // several cycles at once is the rarer thing and it chains:
    //
    //     tune | Zip [Lm 4/3, Lm 2/3] | Zip [Gm 1, Gm 1/2, Gm 3/4]
    //
    // The pattern normalizes against a UNIT form, never against `input`.
    // Zipping it against the input too would multiply the subject's own fm
    // and length back into the result.
    let mut pattern = NormalForm::init();
    crate::ast::Op::Sequence {
        operations: operations.to_vec(),
    }
    .apply_to_normal_form(&mut pattern, defs)?;
    let patterns = [pattern];

    // A single-event subject is almost always a mistake: `Zip [a, b]` written
    // standalone, where the intended subject is sitting in the bracket list
    // instead of arriving through the pipe. Say so — the result would be one
    // note, and nothing else would explain why.
    let subject_events = subject.operations.iter().map(|v| v.len()).max().unwrap_or(0);
    let pattern_events = patterns
        .iter()
        .filter_map(|p| p.operations.iter().map(|v| v.len()).max())
        .max()
        .unwrap_or(0);
    if subject_events <= 1 && pattern_events > 1 {
        println!(
            "{} Zip's subject has {} event(s) but a pattern has {}. Zip multiplies \
             patterns into the events the SUBJECT already has, and the subject comes \
             through the pipe: `melody | Zip [rhythm]`, not `Zip [melody, rhythm]`.",
            "[check]".yellow().bold(),
            subject_events,
            pattern_events,
        );
    }

    // THE TALEA DOES NOT CLOSE.
    //
    // A pattern of N steps over a subject of M events leaves the last cycle
    // partial whenever N does not divide M. That is legitimate isorhythm on
    // its own — the phase relationship is the point. It becomes a surprise the
    // moment the zipped material is REPEATED, because `Repeat` copies what the
    // zip already produced, so the talea restarts at step 0 in every copy and
    // the seam gets two of the same step back to back:
    //
    //     Seq [5 notes] | Zip [Lm 11/20, Lm 9/20] | Repeat 2
    //     0.55 0.45 0.55 0.45 0.55 │ 0.55 0.45 …
    //                              ^ two longs, at the phrase boundary
    //
    // Repeating BEFORE the zip runs one talea across the whole thing and the
    // seam disappears. That is a real compositional choice, not a bug, so this
    // says what happened rather than changing it.
    if pattern_events > 1 && subject_events > 1 && subject_events % pattern_events != 0 {
        println!(
            "{} Zip's talea has {} steps but the subject has {} events, so the \
             cycle does not close ({} left over). Fine on its own — but a \
             `Repeat` AFTER this zip restarts the talea each copy, which puts \
             two of the same step together at the seam. `| Repeat n | Zip [...]` \
             runs one talea across the whole phrase instead.",
            "[check]".yellow().bold(),
            pattern_events,
            subject_events,
            subject_events % pattern_events,
        );
    }

    for voice in subject.operations.iter_mut() {
        for (i, point) in voice.iter_mut().enumerate() {
            for pattern in &patterns {
                let Some(pattern_voice) = pattern.operations.first() else {
                    continue;
                };
                if pattern_voice.is_empty() {
                    continue;
                }
                // PointOp's own `Mul`: fm/pm/g/l multiply, fa/pa add, names
                // union. Non-multiplicative fields (osc_type, asr, is_out)
                // take the RIGHT operand, so a later pattern in the list wins
                // for those — the same precedence as writing it later in a pipe.
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
