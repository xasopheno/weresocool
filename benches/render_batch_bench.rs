use criterion::{black_box, criterion_group, criterion_main, Criterion};

use weresocool::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
};
use weresocool_instrument::{
    renderable::{nf_to_vec_renderable, render_voice::renderables_to_render_voices},
    StereoWaveform,
};

fn render_batch_bench(c: &mut Criterion) {
    let filename = "songs/test/render_op_get_batch.socool".to_string();
    let (nf, basis, table) = match Filename(&filename)
        .make(RenderType::NfBasisAndTable)
        .unwrap()
    {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => {
            panic!();
        }
    };

    let renderables = nf_to_vec_renderable(&nf, &table, &basis).unwrap();
    let mut voices1 = renderables_to_render_voices(renderables);
    c.bench_function("render_batch", |b| {
        b.iter(|| {
            let _r: Vec<StereoWaveform> = voices1
                .iter_mut()
                .map(|voice| voice.render_batch(black_box(1024), None).unwrap())
                .collect();
        })
    });
}

criterion_group!(benches, render_batch_bench);
criterion_main!(benches);
