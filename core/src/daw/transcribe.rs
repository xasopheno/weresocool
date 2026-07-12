//! Turn a recorded loop (WAV) into a `NormalForm` of note events.
//!
//! The analyzer is YIN for now (pluggable later — the raw audio is kept so a
//! different analyzer can be swapped in without re-recording). We reuse
//! weresocool's `from_sound_yin_to_normalform`, which emits one note per frame,
//! then **run-length-merge** consecutive same-pitch frames into single held
//! notes so the resulting def is compact instead of one-note-per-frame.

use std::path::Path;
use weresocool_ast::datagen::from_sound::from_sound_yin_to_normalform;
use weresocool_ast::{NormalForm, PointOp};
use num_rational::Rational64;

/// Transcribe a recorded WAV into a merged `NormalForm` suitable for seeding
/// the `Perform` recordings registry.
pub fn transcribe_wav(wav_path: &Path, fps: usize) -> Result<NormalForm, String> {
    let path = wav_path
        .to_str()
        .ok_or_else(|| "non-utf8 wav path".to_string())?;
    let nf = from_sound_yin_to_normalform(path, fps).map_err(|e| e.to_string())?;
    Ok(merge_held_notes(nf))
}

/// Collapse runs of consecutive note-ops that share the same pitch (and
/// voiced/silent status) into a single held note whose length is the sum of the
/// run and whose gain is the run's mean. The onset op's attack is preserved so
/// the held note keeps a single attack at its start.
pub fn merge_held_notes(nf: NormalForm) -> NormalForm {
    let merged: Vec<Vec<PointOp>> = nf
        .operations
        .into_iter()
        .map(merge_voice)
        .collect();
    NormalForm {
        operations: merged,
        ..nf
    }
}

fn merge_voice(ops: Vec<PointOp>) -> Vec<PointOp> {
    let mut out: Vec<PointOp> = Vec::with_capacity(ops.len());
    // Parallel run-lengths so we can mean the gain when a run closes.
    let mut counts: Vec<i64> = Vec::new();
    for op in ops {
        match out.last_mut() {
            Some(prev) if same_note(prev, &op) => {
                prev.l += op.l; // held note length grows
                prev.g += op.g; // summed now, meaned below
                *counts.last_mut().unwrap() += 1;
            }
            _ => {
                out.push(op);
                counts.push(1);
            }
        }
    }
    for (op, count) in out.iter_mut().zip(counts) {
        if count > 1 {
            op.g /= Rational64::from_integer(count);
        }
    }
    out
}

/// Pitches within this many cents are treated as the same held note. YIN
/// returns a slightly different frequency every frame (vibrato, noise), so
/// merging needs a tolerance — exact equality would never merge real singing
/// and you'd get one note per frame. ~70 cents (just over a quarter tone)
/// folds vibrato into one note without swallowing a real semitone step.
const MERGE_CENTS: f64 = 70.0;

/// Whether two ops belong to the same held note: both voiced and within
/// `MERGE_CENTS` of each other, or both silent. Silence is gain == 0.
fn same_note(a: &PointOp, b: &PointOp) -> bool {
    let a_silent = a.g == Rational64::from_integer(0);
    let b_silent = b.g == Rational64::from_integer(0);
    if a_silent || b_silent {
        return a_silent && b_silent;
    }
    cents_between(a.fm, b.fm) <= MERGE_CENTS
}

/// Absolute pitch distance in cents between two frequency multipliers.
fn cents_between(a: Rational64, b: Rational64) -> f64 {
    use num_traits::ToPrimitive;
    let (a, b) = (a.to_f64().unwrap_or(0.0), b.to_f64().unwrap_or(0.0));
    if a <= 0.0 || b <= 0.0 {
        return f64::INFINITY;
    }
    1200.0 * (a / b).log2().abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use weresocool_ast::OscType;

    fn note(fm: Rational64, g: Rational64, l: Rational64) -> PointOp {
        PointOp {
            fm,
            g,
            l,
            osc_type: OscType::Sine { pow: None },
            ..Default::default()
        }
    }

    // Approximate a frequency multiplier as a rational, like the YIN path does.
    fn fm(x: f64) -> Rational64 {
        Rational64::approximate_float(x).unwrap_or_else(|| Rational64::from_integer(1))
    }

    #[test]
    fn merges_jittered_held_note() {
        let frame = Rational64::new(1, 30);
        let g = Rational64::new(1, 2);
        // A held note with realistic per-frame pitch jitter (±~15 cents),
        // then a clearly different note (a major third up), then silence.
        let ops = vec![
            note(fm(1.000), g, frame),
            note(fm(1.006), g, frame), // ~10 cents — same note
            note(fm(0.993), g, frame), // ~12 cents — same note
            note(fm(1.260), g, frame), // ~400 cents — new note
            note(fm(1.0), Rational64::from_integer(0), frame),
            note(fm(1.0), Rational64::from_integer(0), frame),
        ];
        let nf = NormalForm {
            operations: vec![ops],
            length_ratio: frame * 6,
            start_at: None,
        };

        let merged = merge_held_notes(nf);
        let voice = &merged.operations[0];
        // jittered run → 1, third → 1, silence → 1  ==> 3 ops
        assert_eq!(voice.len(), 3, "jittered pitches should merge into one note");
        assert_eq!(voice[0].l, frame * 3); // held note spans its 3 frames
        assert_eq!(voice[2].g, Rational64::from_integer(0));
        assert_eq!(voice[2].l, frame * 2); // merged silence
    }
}
