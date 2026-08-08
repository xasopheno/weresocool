//! THE BLOCK SIZE MUST NOT CHANGE THE SOUND.
//!
//! Block size is a property of the audio device (live), or of
//! `Settings::buffer_size` (offline batching), or of nothing at all (the
//! snapshot corpus renders each note whole). It is never a property of the
//! music. A note rendered as one 8000-sample op and the same note rendered as
//! 63 chunks of 128 must produce the same samples.
//!
//! This did not hold. `Voice::calculate_op_gain` ran once per buffer with the
//! index at the buffer's END, so the ASR envelope was a staircase sampled at
//! the block rate — and on the unchunked path, where the buffer IS the note,
//! it was evaluated exactly once, at `index == total_length`, which made the
//! declared attack branch unreachable by one sample. The envelope now runs
//! inside the per-sample loop on `op.sample_index() + i`, the same note-scoped
//! clock the drum oscillators always used, so this is true by construction.
//!
//! WHY THIS FILE IS WRITTEN CAREFULLY: an earlier version built its test note
//! with `RenderOp::init_fglps`, which hardcodes `attack: 512.0`
//! (`renderable/mod.rs:176`). Real notes never get that. `create_render_ops`
//! sets `attack: r_to_f64(point_op.attack * basis.a) * sample_rate`, and
//! `basis.a` is ALWAYS 1/1 (a `.socool` file has no way to set it), with
//! `PointOp`'s default attack also 1/1 — so a real note's attack is ONE
//! SECOND. That decides which branch `asr.rs` takes: `is_short` is
//! `total <= attack + decay`, so every note under two seconds has
//! `attack_length` rewritten to the whole note. Measuring the wrong one
//! produced a confident, wrong diagnosis. So: build ops the way
//! `create_render_ops` does.

use weresocool_instrument::renderable::{RenderOp, Renderable};
use weresocool_instrument::Oscillator;
use weresocool_shared::Settings;

/// Build a note the way `create_render_ops` does — the point of this file.
/// `seconds` is the note's length; attack and decay are one second, which is
/// what `PointOp`'s defaults × `basis.a == 1/1` actually produce.
fn real_note(hz: f64, seconds: f64) -> RenderOp {
    let sr = Settings::global().sample_rate;
    let n = (seconds * sr).round() as usize;
    let mut op = RenderOp::init_fglps(hz, (1.0, 1.0), seconds, 0.0, n);
    op.total_samples = n;
    op.attack = sr; // point_op.attack (1/1) * basis.a (1/1) * sample_rate
    op.decay = sr;
    op.gain_scalar = 1.0;
    op
}

/// Slice a note into buffers the way `RenderVoice::get_batch` does: `samples`
/// becomes the BUFFER length, `index` the offset of that buffer in the note,
/// and `total_samples` stays the note.
fn chunk(op: &RenderOp, block: usize) -> Vec<RenderOp> {
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < op.total_samples {
        let n = block.min(op.total_samples - at);
        out.push(RenderOp { samples: n, index: at, ..op.clone() });
        at += n;
    }
    out
}

fn render(mut ops: Vec<RenderOp>) -> Vec<f64> {
    let mut osc = Oscillator::init();
    ops.render(&mut osc, None).l_buffer
}

fn worst_diff(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "renders disagree on length, not just content");
    a.iter().zip(b.iter()).fold(0.0f64, |m, (x, y)| m.max((x - y).abs()))
}

/// The four block sizes that actually occur: kintaro watch and print (128,
/// unified in kintaro `52febb1`), an arbitrary other device block, the
/// weresocool CLI's `Settings::buffer_size`, and the snapshot corpus, which
/// renders each note whole.
#[test]
fn block_size_does_not_change_the_sound() {
    Settings::init_test();

    // lulupea's actual scale: thing1's notes measured 0.167 s and 0.5 s. The
    // 3 s case is past the `is_short` threshold (attack + decay = 2 s), so the
    // other branch of `calculate_long_gain` is covered too — that case used to
    // diverge by 0.66 between whole and chunked.
    for (label, hz, secs) in [
        ("55 Hz, 0.167 s", 55.0, 0.167),
        ("55 Hz, 0.5 s", 55.0, 0.5),
        ("440 Hz, 0.167 s", 440.0, 0.167),
        ("55 Hz, 3 s", 55.0, 3.0),
    ] {
        let op = real_note(hz, secs);
        let reference = render(vec![op.clone()]); // whole — the corpus path

        for block in [128usize, 256, 12288] {
            let diff = worst_diff(&reference, &render(chunk(&op, block)));
            assert!(
                diff < 1e-9,
                "{label}: block size {block} changed the sound (worst sample \
                 difference {diff:.6}). The envelope is reading something that \
                 belongs to the buffer instead of to the note.",
            );
        }
    }
}

/// A note whose gain the envelope has to ramp INTO — the previous note's
/// ending gain is the attack's starting point, and that anchor is latched once
/// per note. If it were re-latched per buffer, chunked and whole would part
/// company here even though the single-note test above passed.
#[test]
fn block_size_does_not_change_a_sequence() {
    Settings::init_test();

    let notes = [
        real_note(55.0, 0.25),
        real_note(110.0, 0.25),
        real_note(55.0, 0.5),
    ];

    let reference: Vec<f64> = notes.iter().flat_map(|n| render(vec![n.clone()])).collect();

    for block in [128usize, 256, 12288] {
        let chunked: Vec<f64> = notes.iter().flat_map(|n| render(chunk(n, block))).collect();
        let diff = worst_diff(&reference, &chunked);
        assert!(
            diff < 1e-9,
            "a three-note sequence rendered at block size {block} differs from \
             the same notes rendered whole (worst sample difference {diff:.6})",
        );
    }
}
