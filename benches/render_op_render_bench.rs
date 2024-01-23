mod perf;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use weresocool_instrument::renderable::Offset;
use weresocool_instrument::renderable::Renderable;
use weresocool_instrument::Oscillator;
use weresocool_instrument::RenderOp;
use weresocool_shared::Settings;

pub fn render_op_render_bench(c: &mut Criterion) {
    Settings::init_default();
    let mut oscillator = Oscillator::init();
    let render_op = RenderOp::init_fglp(400.0, (1.0, 1.0), 10.0, 0.0, Settings::global());

    let offset = Offset {
        freq: 1.0,
        gain: 1.0,
    };

    c.bench_function("render_op_render_bench", |b| {
        b.iter(|| black_box(render_op.render(&mut oscillator, Some(&offset))))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = render_op_render_bench
}
criterion_main!(benches);
