use weresocool_core::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    portaudio::duplex_setup,
    ui::were_so_cool_logo,
};
use weresocool_error::Error;
use weresocool_instrument::renderable::nf_to_vec_renderable;
use weresocool_shared::r_to_f64;

fn main() -> Result<(), Error> {
    were_so_cool_logo(None, None);
    println!("       )))***=== REAL<GOOD>TIME *mic ===***(((  \n ");

    let filename = "test.socool";

    let (nf, basis, mut table) = match Filename(filename).make(RenderType::NfBasisAndTable, None)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;

    println!("\nGenerating Composition ");
    let mut duplex_stream = duplex_setup(r_to_f64(basis.f), renderables)?;
    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
