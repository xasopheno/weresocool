use criterion::{black_box, criterion_group, criterion_main, Criterion};

use weresocool::{
    generation::{filename_to_render, RenderReturn, RenderType},
    instrument::StereoWaveform,
    renderable::{nf_to_vec_renderable, render_voice::renderables_to_render_voices},
};

fn criterion_benchmark(c: &mut Criterion) {
    let filename = "songs/test/render_op_get_batch.socool".to_string();
    let (nf, basis, table) =
        match filename_to_render(&filename, RenderType::NfBasisAndTable).unwrap() {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => {
                panic!();
            }
        };

    let renderables = nf_to_vec_renderable(&nf, &table, &basis);
    let mut voices1 = renderables_to_render_voices(renderables);
    c.bench_function("render_batch", |b| {
        b.iter(|| {
            let _r: Vec<StereoWaveform> = voices1
                .iter_mut()
                .map(|voice| voice.render_batch(black_box(1024), None))
                .collect();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
