//! Honest measurement of the audio-render thread pool's parallel
//! scaling.
//!
//! Runs the same render scenario at varying audio thread counts and
//! reports wall-clock time per iteration. Pair with `/usr/bin/time -l`
//! to see total CPU time (user+sys) vs wall time — the ratio reveals
//! whether parallelism is delivering real wall-clock gains or just
//! burning more cores for the same total work.
//!
//! Usage:
//!     # voice_count thread_count
//!     /usr/bin/time -l cargo run --release --example parallel_scaling -p weresocool_core -- 100 1
//!     /usr/bin/time -l cargo run --release --example parallel_scaling -p weresocool_core -- 100 4
//!     /usr/bin/time -l cargo run --release --example parallel_scaling -p weresocool_core -- 100 8
//!     /usr/bin/time -l cargo run --release --example parallel_scaling -p weresocool_core -- 100 16
//!
//! Both args are optional: defaults are 100 voices, current setting's
//! audio_thread_count. Thread count is configured by writing it into
//! Settings BEFORE `Settings::init` so the audio pool picks it up on
//! first use.

use std::time::Instant;
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

fn render_once(voices: usize, ops_per_voice: usize, reads: usize, buffer: usize, sample_rate: f64) {
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

    let mut produced = 0;
    while produced < reads {
        match rm.read(buffer, Offset::default()) {
            Some((sw, _ramp, _ops)) => {
                std::hint::black_box(sw);
                produced += 1;
            }
            None => break,
        }
    }
}

fn main() {
    let voices: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let thread_count: Option<usize> = std::env::args().nth(2).and_then(|s| s.parse().ok());

    let buffer = 1024usize;
    let sample_rate = 48_000.0;

    // Configure Settings BEFORE the audio pool is first touched. The
    // audio pool reads `audio_thread_count` once at init.
    let mut s = weresocool_shared::default_settings();
    s.sample_rate = sample_rate;
    s.buffer_size = buffer;
    if let Some(n) = thread_count {
        s.audio_thread_count = n;
        // For the sweep, drop the voice-count threshold so even small
        // voice counts go through the pool — otherwise low-voice-count
        // / low-thread-count rows wouldn't exercise parallelism.
        s.parallel_voice_threshold = 1;
    }
    s.set();

    let ops_per_voice = 4;
    let reads = 16;
    let iterations = 50;

    println!(
        "voices={voices} ops_per_voice={ops_per_voice} reads={reads} iters={iterations}"
    );
    println!(
        "audio_thread_count={}  parallel_voice_threshold={}",
        Settings::global().audio_thread_count,
        Settings::global().parallel_voice_threshold
    );

    // Warmup
    for _ in 0..5 {
        render_once(voices, ops_per_voice, reads, buffer, sample_rate);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        render_once(voices, ops_per_voice, reads, buffer, sample_rate);
    }
    let elapsed = start.elapsed();

    let per_iter_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    println!(
        "wall total: {:.3} s   per iteration: {:.3} ms",
        elapsed.as_secs_f64(),
        per_iter_ms
    );
}
