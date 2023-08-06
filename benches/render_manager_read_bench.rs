mod perf;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use weresocool_core::manager::RenderManager;

use weresocool::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
};
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, render_voice::renderables_to_render_voices,
};

use weresocool_shared::Settings;

fn setup_render_manager() -> RenderManager {
    let filename = "simple.socool";
    let mut rm = RenderManager::init(None, None, true, None);
    let (nf, basis, mut table) = match Filename(&filename)
        .make(RenderType::NfBasisAndTable, None)
        .unwrap()
    {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => {
            panic!();
        }
    };

    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis).unwrap();
    let voices = renderables_to_render_voices(renderables);

    rm.push_render(voices, true);

    rm
}

fn read_benchmark(c: &mut Criterion) {
    Settings::init_default();
    let mut manager = setup_render_manager();
    let buffer_size = 1024 * 12;

    c.bench_function("render_manager_read_bench", |b| {
        b.iter(|| {
            let manager = black_box(&mut manager);
            manager.read(buffer_size)
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = read_benchmark
}
criterion_main!(benches);
