//! DIAGNOSTIC: does the audio block size change how a note sounds?
//!
//! It must not. Block size is a property of the audio device (live) or of
//! `Settings::buffer_size` (offline batching), never of the music. But the
//! sustained-tone gain ramp is measured against `RenderOp.samples`, which
//! `render_voice.rs` sets to `samples_left_in_batch` — the BLOCK, not the
//! note.
//!
//! WHY THIS FILE IS WRITTEN CAREFULLY: an earlier version of it built its
//! test note with `RenderOp::init_fglps`, which hardcodes `attack: 512.0`
//! (`renderable/mod.rs:176`). Real notes never get that. `create_render_ops`
//! sets `attack: r_to_f64(point_op.attack * basis.a) * sample_rate`, and
//! `basis.a` is ALWAYS 1/1 (a `.socool` file has no way to set it), with
//! `PointOp`'s default attack also 1/1 — so a real note's attack is ONE
//! SECOND.
//!
//! That difference decides which branch `asr.rs` takes:
//! `is_short(total, attack, decay)` is `total <= attack + decay`, so with a
//! one-second attack every note under two seconds takes the `short` branch
//! and `attack_length` is rewritten to the whole note. With a 512-sample
//! attack it does not. Measuring the wrong one produced a confident, wrong
//! diagnosis. So: build ops the way `create_render_ops` does.

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

fn chunk(op: &RenderOp, block: usize) -> Vec<RenderOp> {
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < op.samples {
        let n = block.min(op.samples - at);
        out.push(RenderOp { samples: n, index: at, ..op.clone() });
        at += n;
    }
    out
}

fn render(mut ops: Vec<RenderOp>) -> Vec<f64> {
    let mut osc = Oscillator::init();
    ops.render(&mut osc, None).l_buffer
}

/// Peak in the first `ms` — how fast the note arrives.
fn onset_peak(buf: &[f64], ms: f64) -> f64 {
    let sr = Settings::global().sample_rate;
    let n = ((sr * ms / 1000.0) as usize).min(buf.len());
    buf[..n].iter().fold(0.0f64, |a, x| a.max(x.abs()))
}

fn report(label: &str, op: &RenderOp) -> Vec<(String, Vec<f64>)> {
    let mut rows = Vec::new();
    let marks = [5.0, 20.0, 50.0, 150.0];
    for (name, buf) in [
        ("watch 128", render(chunk(op, 128))),
        ("print 256", render(chunk(op, 256))),
        ("cli 12288", render(chunk(op, 12288))),
        ("unchunked", render(vec![op.clone()])),
    ] {
        let peaks: Vec<f64> = marks.iter().map(|m| onset_peak(&buf, *m)).collect();
        println!(
            "  {label:<22} {name:<10} @5ms={:.4} @20ms={:.4} @50ms={:.4} @150ms={:.4}",
            peaks[0], peaks[1], peaks[2], peaks[3]
        );
        rows.push((name.to_string(), buf));
    }
    rows
}

fn worst_diff(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).fold(0.0f64, |m, (x, y)| m.max((x - y).abs()))
}

#[test]
#[ignore = "DIAGNOSTIC, not a regression. Run with --ignored to print the \
onset tables. Whether this is a real defect depends on the numbers it prints \
for REAL notes (attack = 1s), not for the synthetic 512-sample-attack op an \
earlier version of this test used."]
fn how_much_does_block_size_change_a_note() {
    Settings::init_test();
    println!();

    // lulupea's actual scale: thing1's notes measured 0.167 s and 0.5 s.
    // A low note and a high one, because the report is that the LOW end
    // suffers.
    for (label, hz, secs) in [
        ("55 Hz, 0.167 s", 55.0, 0.167),
        ("55 Hz, 0.5 s", 55.0, 0.5),
        ("440 Hz, 0.167 s", 440.0, 0.167),
        // Past the `is_short` threshold (attack + decay = 2 s), so the OTHER
        // branch of `calculate_long_gain` is covered.
        ("55 Hz, 3 s", 55.0, 3.0),
    ] {
        let op = real_note(hz, secs);
        let rows = report(label, &op);
        let watch = &rows[0].1;
        for (name, buf) in rows.iter().skip(1) {
            println!(
                "  {:<22} {:<10} worst difference vs watch: {:.4}",
                "", name, worst_diff(watch, buf)
            );
        }
        println!();
    }
}
