use criterion::{criterion_group, criterion_main, Criterion};

use weresocool::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
};

use weresocool_instrument::renderable::nf_to_vec_renderable;

fn nf_to_vec_renderable_bench(c: &mut Criterion) {
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

    c.bench_function("nf_to_vec_renderable_bench", |b| {
        b.iter(|| {
            let _renderables = nf_to_vec_renderable(&nf, &table, &basis);
        })
    });
}

criterion_group!(benches, nf_to_vec_renderable_bench);
criterion_main!(benches);
