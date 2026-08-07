//! DIAGNOSTIC: does the audio buffer size change how a note sounds?
//!
//! It must not. Buffer size is a property of the audio device (live) or of
//! `Settings::buffer_size` (offline batching, 12288), never of the music.
//! But the sustained-tone gain ramp is measured against `RenderOp.samples`,
//! which `render_voice.rs` sets to `samples_left_in_batch` — the BUFFER, not
//! the note. So the same note gets a ~11 ms attack through a 512-sample
//! device buffer and a ~256 ms attack through a 12288-sample print batch.
//!
//! That is the difference between `kintaro watch` and `kintaro print`.

use weresocool_instrument::renderable::{RenderOp, Renderable};
use weresocool_instrument::Oscillator;
use weresocool_shared::Settings;

fn chunk(op: &RenderOp, buffer: usize) -> Vec<RenderOp> {
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < op.samples {
        let n = buffer.min(op.samples - at);
        out.push(RenderOp {
            samples: n,
            index: at,
            ..op.clone()
        });
        at += n;
    }
    out
}

fn render(mut ops: Vec<RenderOp>) -> Vec<f64> {
    let mut osc = Oscillator::init();
    ops.render(&mut osc, None).l_buffer
}

/// Peak amplitude in the first `ms` milliseconds — how fast the note arrives.
fn onset_peak(buf: &[f64], ms: f64) -> f64 {
    let sr = Settings::global().sample_rate;
    let n = ((sr * ms / 1000.0) as usize).min(buf.len());
    buf[..n].iter().fold(0.0f64, |a, x| a.max(x.abs()))
}

#[test]
#[ignore = "KNOWN DEFECT, not a regression: a note's attack is currently the audio buffer's length. Run with --ignored to see the table. Fixing it means making the envelope note-relative, which changes how every piece sounds."]
fn buffer_size_must_not_change_a_notes_attack() {
    Settings::init_test();
    let sr = Settings::global().sample_rate as usize;

    // A LOW note — 55 Hz — held for one second. Low frequencies are where
    // this is reported to be audible, and they are also where a smeared
    // attack is most obvious: the ramp lasts many cycles.
    let mut op = RenderOp::init_fglps(55.0, (1.0, 1.0), 1.0, 0.0, sr);
    op.total_samples = sr;
    op.gain_scalar = 1.0;

    // 512  — a typical live audio device buffer (`kintaro watch`)
    // 12288 — `Settings::buffer_size`, what offline batching uses (`print`)
    let live = render(chunk(&op, 512));
    let print = render(chunk(&op, 12288));
    let whole = render(vec![op.clone()]);

    for (label, buf) in [("live 512", &live), ("print 12288", &print), ("whole", &whole)] {
        println!(
            "{label:>12}: peak@10ms={:.4} peak@50ms={:.4} peak@250ms={:.4} peak@full={:.4}",
            onset_peak(buf, 10.0),
            onset_peak(buf, 50.0),
            onset_peak(buf, 250.0),
            onset_peak(buf, 1000.0),
        );
    }

    let worst = live
        .iter()
        .zip(print.iter())
        .fold(0.0f64, |a, (x, y)| a.max((x - y).abs()));
    println!("   worst sample difference live vs print: {worst:.4}");

    assert!(
        worst < 1e-6,
        "the same note sounds different at 512 and 12288 samples per buffer \
         (worst difference {worst}). Buffer size is a property of the audio \
         device, not of the music."
    );
}
