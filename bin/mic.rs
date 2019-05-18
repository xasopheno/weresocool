use socool_ast::api::{
    NormalForm, Normalize
};
use socool_parser::parser::parse_file;
use weresocool::{
    examples::documentation,
    portaudio_setup::duplex::setup_portaudio_duplex,
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

    let parsed = parse_file(&filename.unwrap().to_string(), None);
    let main = parsed.table.get("main").unwrap();

    let mut normal_form = NormalForm::init();

    println!("\nGenerating Composition ");
    main.apply_to_normal_form(&mut normal_form, &parsed.table);

    let pa = pa::PortAudio::new()?;

    let mut duplex_stream = setup_portaudio_duplex(normal_form.operations, &pa)?;
    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
