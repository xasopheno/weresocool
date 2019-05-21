use socool_ast::{NormalForm, Normalize};
use socool_parser::parse_file;
use weresocool::{
    examples::documentation,
    generation::{filename_to_render, RenderReturn, RenderType},
    portaudio::duplex_setup,
    ui::{get_args, no_file_name, were_so_cool_logo},
};

use portaudio as pa;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    were_so_cool_logo();
    println!("{}", "       )))***=== MICROPHONE ===***(((  \n ");

    let args = get_args();

    if args.is_present("doc") {
        documentation();
    }

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let mut normal_form = match filename_to_render(filename.unwrap(), RenderType::NfAndBasis) {
        RenderReturn::NfAndBasis(nf, basis) => nf,
        _ => panic!("Error. Unable to generate NormalForm"),
    };

    println!("\nGenerating Composition ");
    main.apply_to_normal_form(&mut normal_form, &parsed.table);

    let mut duplex_stream = duplex_setup(normal_form.operations)?;
    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
