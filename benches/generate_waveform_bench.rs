mod perf;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use weresocool_instrument::renderable::Offset;
use weresocool_instrument::voice::Voice;
use weresocool_instrument::RenderOp;
use weresocool_shared::Settings;

fn setup_op_and_offset() -> RenderOp {
    Settings::init_default();
    let op = RenderOp::init_fglp(400.0, (1.0, 1.0), 10.0, 0.0, Settings::global());
    op
}

fn generate_waveform_benchmark(c: &mut Criterion) {
    let op = setup_op_and_offset();
    let mut voice = Voice::init(0);

    c.bench_function("generate_waveform_bench", |b| {
        b.iter(|| {
            let op = black_box(&op);
            voice.generate_waveform(
                op,
                &Offset {
                    freq: 1.0,
                    gain: 1.0,
                },
            )
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = generate_waveform_benchmark
}
criterion_main!(benches);
