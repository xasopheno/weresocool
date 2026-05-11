//! Determinism check for the parallel voice-render path.
//!
//! With rayon, voices are rendered in work-stealing order, but the
//! orchestrator collects per-voice results in original voice-index
//! order before merging. This test renders the same input twice
//! through the parallel path (voice count above PARALLEL_VOICE_THRESHOLD)
//! and asserts the L/R sample buffers are bit-identical across runs.
//!
//! If this ever fails, the parallel path is leaking iteration order
//! into the final f64 sum and needs to be fixed before further use.

use weresocool_core::manager::{RenderManager, RenderManagerSettings};
use weresocool_instrument::renderable::{render_voice::RenderVoice, RenderOp};
use weresocool_instrument::Offset;
use weresocool_shared::Settings;

fn make_ops(num_ops: usize, samples_per_op: usize, sample_rate: f64) -> Vec<RenderOp> {
    let mut ops = Vec::with_capacity(num_ops);
    for i in 0..num_ops {
        let f = 220.0 + (i % 8) as f64 * 15.0;
        let g = (0.2, 0.2);
        let p = 0.0;
        let l = samples_per_op as f64 / sample_rate;
        let mut op = RenderOp::init_fglps(f, g, l, p, samples_per_op);
        op.index = 0;
        op.total_samples = samples_per_op;
        ops.push(op);
    }
    ops
}

fn render_one(voices: usize, ops_per_voice: usize, reads: usize) -> (Vec<f64>, Vec<f64>) {
    Settings::init(48_000.0, 1024);
    let buffer = 1024usize;
    let sample_rate = 48_000.0;
    let settings = RenderManagerSettings { sample_rate, buffer_size: buffer };
    let mut rm = RenderManager::init(None, None, false, Some(settings));

    let mut all_voices: Vec<RenderVoice> = Vec::with_capacity(voices);
    for v in 0..voices {
        let mut ops = make_ops(ops_per_voice, buffer * reads, sample_rate);
        for (event, op) in ops.iter_mut().enumerate() {
            op.voice = v;
            op.event = event;
        }
        all_voices.push(RenderVoice::init(&ops));
    }
    rm.push_render(all_voices, false);

    let mut left: Vec<f64> = Vec::with_capacity(reads * buffer);
    let mut right: Vec<f64> = Vec::with_capacity(reads * buffer);
    let mut produced = 0;
    while produced < reads {
        match rm.read(buffer, Offset::default()) {
            Some((sw, _ramp, _ops)) => {
                left.extend_from_slice(&sw.l_buffer);
                right.extend_from_slice(&sw.r_buffer);
                produced += 1;
            }
            None => break,
        }
    }
    (left, right)
}

#[test]
fn parallel_render_is_deterministic_at_100_voices() {
    let (l1, r1) = render_one(100, 4, 4);
    let (l2, r2) = render_one(100, 4, 4);

    assert_eq!(l1.len(), l2.len(), "L buffer length differs across runs");
    assert_eq!(r1.len(), r2.len(), "R buffer length differs across runs");

    for (i, (a, b)) in l1.iter().zip(l2.iter()).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "L mismatch at sample {i}: run1={a} run2={b}"
        );
    }
    for (i, (a, b)) in r1.iter().zip(r2.iter()).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "R mismatch at sample {i}: run1={a} run2={b}"
        );
    }
}

#[test]
fn parallel_render_is_deterministic_at_200_voices() {
    let (l1, r1) = render_one(200, 4, 2);
    let (l2, r2) = render_one(200, 4, 2);

    assert_eq!(l1, l2, "L buffer differs across runs of 200-voice render");
    assert_eq!(r1, r2, "R buffer differs across runs of 200-voice render");
}
