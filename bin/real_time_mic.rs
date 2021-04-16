use weresocool::{
    examples::documentation,
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    portaudio::duplex_setup,
    ui::{get_args, no_file_name, were_so_cool_logo},
};
use weresocool_error::Error;
use weresocool_instrument::renderable::nf_to_vec_renderable;
use weresocool_shared::r_to_f64;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    println!("       )))***=== REAL<GOOD>TIME *mic ===***(((  \n ");

    let args = get_args();

    if args.is_present("doc") {
        documentation();
    }

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let (nf, basis, table) =
        match Filename(filename.unwrap()).make(RenderType::NfBasisAndTable, None)? {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => panic!("Error. Unable to generate NormalForm"),
        };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;

    println!("\nGenerating Composition ");
    let mut duplex_stream = duplex_setup(r_to_f64(basis.f), renderables)?;
    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
