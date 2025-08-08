use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use weresocool_core::generation::sum_all_waveforms;
use weresocool_core::generation::sum_vec;
use weresocool_core::manager::{RenderManager, RenderManagerSettings};
use weresocool_instrument::renderable::{render_voice::RenderVoice, Offset, RenderOp, Renderable};
use weresocool_instrument::{Oscillator, StereoWaveform};
use weresocool_shared::Settings;

fn make_ops(num_ops: usize, samples_per_op: usize, sample_rate: f64) -> Vec<RenderOp> {
    let mut ops = Vec::with_capacity(num_ops);
    for i in 0..num_ops {
        // Sweep frequencies a bit; non-zero gains
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

fn bench_vec_renderop_render(c: &mut Criterion) {
    // Ensure global settings are initialized for instrument code paths
    Settings::init(48_000.0, 1024);
    let mut group = c.benchmark_group("vec_renderop_render");
    let sample_rate = 48_000.0;
    let buffer = 1024usize;

    for &ops_count in &[4usize, 16, 64] {
        group.throughput(Throughput::Elements((ops_count * buffer) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(ops_count), &ops_count, |b, &n| {
            b.iter_batched(
                || {
                    let mut osc = Oscillator::init();
                    let mut ops = make_ops(n, buffer, sample_rate);
                    (osc, ops)
                },
                |(mut osc, mut ops)| {
                    let _sw: StereoWaveform = ops.render(&mut osc, Some(&Offset::default()));
                    criterion::black_box(_sw);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_render_manager_read(c: &mut Criterion) {
    // Ensure global settings are initialized for render manager
    Settings::init(48_000.0, 1024);
    let mut group = c.benchmark_group("render_manager_read");
    let sample_rate = 48_000.0;
    let buffer = 1024usize;

    for &(voices, ops_per_voice, reads) in &[(2usize, 8usize, 256usize), (8, 8, 256), (8, 32, 128)] {
        let id = format!("v{}_o{}_r{}", voices, ops_per_voice, reads);
        group.throughput(Throughput::Bytes((reads * buffer * 2 * 4) as u64));
        group.bench_function(BenchmarkId::from_parameter(id), |b| {
            b.iter_batched(
                || {
                    let settings = RenderManagerSettings { sample_rate, buffer_size: buffer };
                    let mut rm = RenderManager::init(None, None, false, Some(settings));

                    let mut all_voices: Vec<RenderVoice> = Vec::with_capacity(voices);
                    for v in 0..voices {
                        let mut ops = make_ops(ops_per_voice, buffer * reads, sample_rate);
                        // annotate voice/event ids for realism
                        for (event, op) in ops.iter_mut().enumerate() {
                            op.voice = v;
                            op.event = event;
                        }
                        all_voices.push(RenderVoice::init(&ops));
                    }
                    rm.push_render(all_voices, false);
                    rm
                },
                |mut rm| {
                    let mut produced = 0usize;
                    while produced < reads {
                        if let Some((sw, _ramp, _ops)) = rm.read(buffer, Offset::default()) {
                            produced += 1;
                            criterion::black_box(sw);
                        } else {
                            break;
                        }
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_vec_renderop_render_old_vs_new(c: &mut Criterion) {
    // Compare the historic clone-heavy loop vs current in-place loop
    Settings::init(48_000.0, 1024);
    let mut group = c.benchmark_group("vec_render_old_vs_new");
    let sample_rate = 48_000.0;
    let buffer = 1024usize;

    fn old_loop(mut osc: Oscillator, ops: &Vec<RenderOp>) -> StereoWaveform {
        let mut result = StereoWaveform::new(0);
        for op in ops.iter() {
            if op.samples > 0 {
                let sw = op.clone().render(&mut osc, Some(&Offset::default()));
                result.append(sw);
            }
        }
        result
    }

    fn new_loop(mut osc: Oscillator, mut ops: Vec<RenderOp>) -> StereoWaveform {
        ops.render(&mut osc, Some(&Offset::default()))
    }

    for &ops_count in &[4usize, 16, 64] {
        group.bench_with_input(BenchmarkId::new("old", ops_count), &ops_count, |b, &n| {
            b.iter_batched(
                || {
                    let osc = Oscillator::init();
                    let ops = make_ops(n, buffer, sample_rate);
                    (osc, ops)
                },
                |(osc, ops)| {
                    let out = old_loop(osc, &ops);
                    criterion::black_box(out);
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("new", ops_count), &ops_count, |b, &n| {
            b.iter_batched(
                || {
                    let osc = Oscillator::init();
                    let ops = make_ops(n, buffer, sample_rate);
                    (osc, ops)
                },
                |(osc, ops)| {
                    let out = new_loop(osc, ops);
                    criterion::black_box(out);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_ramp_build(c: &mut Criterion) {
    // Compare previous map-collect vs current push with capacity
    let mut group = c.benchmark_group("ramp_build");
    for &buf in &[256usize, 1024, 4096, 16384] {
        group.bench_with_input(BenchmarkId::new("old", buf), &buf, |b, &buffer_size| {
            b.iter(|| {
                let past = 0.8f32;
                let current = 0.9f32;
                let distance = current - past;
                let out: Vec<f32> = (0..buffer_size * 2)
                    .map(|i| past + (distance * i as f32 / (buffer_size * 2) as f32))
                    .collect();
                criterion::black_box(out);
            });
        });

        group.bench_with_input(BenchmarkId::new("new", buf), &buf, |b, &buffer_size| {
            b.iter(|| {
                let past = 0.8f32;
                let current = 0.9f32;
                let distance = current - past;
                let mut out: Vec<f32> = Vec::with_capacity(buffer_size * 2);
                let denom = (buffer_size * 2) as f32;
                for i in 0..(buffer_size * 2) {
                    out.push(past + (distance * i as f32 / denom));
                }
                criterion::black_box(out);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_vec_renderop_render, bench_render_manager_read);

fn bench_get_batch(c: &mut Criterion) {
    Settings::init(48_000.0, 1024);
    let mut group = c.benchmark_group("render_voice_get_batch");
    let sample_rate = 48_000.0;

    for &(batch, reads) in &[(1024usize, 512usize), (2048, 256), (4096, 128)] {
        let id = format!("b{}_r{}", batch, reads);
        group.bench_function(BenchmarkId::from_parameter(id), |b| {
            b.iter_batched(
                || {
                    // Single long op to keep slicing fast
                    let total = batch * reads;
                    let mut ops = make_ops(1, total, sample_rate);
                    RenderVoice::init(&ops)
                },
                |mut voice| {
                    let mut remaining = reads;
                    while remaining > 0 {
                        let _ = voice.get_batch(batch, None, false);
                        remaining -= 1;
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn bench_sum_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_vec");
    for &len in &[1024usize, 4096, 16384, 65536] {
        group.bench_with_input(BenchmarkId::from_parameter(len), &len, |b, &n| {
            b.iter_batched(
                || {
                    let mut a = vec![0.0f64; n];
                    let b = vec![1.0f64; n];
                    (a, b)
                },
                |(mut a, b)| {
                    sum_vec(a.as_mut_slice(), &b);
                    criterion::black_box(a);
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_sum_all_waveforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_all_waveforms");
    for &(voices, len) in &[(2usize, 1024usize), (8, 1024), (16, 4096), (32, 4096)] {
        let id = format!("v{}_n{}", voices, len);
        group.bench_function(BenchmarkId::from_parameter(id), |b| {
            b.iter_batched(
                || {
                    let mut inputs = Vec::with_capacity(voices);
                    for _ in 0..voices {
                        inputs.push(StereoWaveform::new(len));
                    }
                    inputs
                },
                |inputs| {
                    let out = sum_all_waveforms(inputs);
                    criterion::black_box(out);
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_oscillator_generate(c: &mut Criterion) {
    Settings::init(48_000.0, 1024);
    let mut group = c.benchmark_group("oscillator_generate");
    let sample_rate = 48_000.0;
    for &len in &[256usize, 1024, 4096] {
        group.bench_with_input(BenchmarkId::from_parameter(len), &len, |b, &n| {
            b.iter_batched(
                || {
                    let mut osc = Oscillator::init();
                    let mut op = RenderOp::init_fglps(440.0, (0.2, 0.2), n as f64 / sample_rate, 0.0, n);
                    (osc, op)
                },
                |(mut osc, mut op)| {
                    osc.update(&op, &Offset::default());
                    let sw = osc.generate(&op, &Offset::default());
                    criterion::black_box(sw);
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(
    extended,
    bench_get_batch,
    bench_sum_vec,
    bench_sum_all_waveforms,
    bench_oscillator_generate,
    bench_vec_renderop_render_old_vs_new,
    bench_ramp_build
);

criterion_main!(benches, extended);
