extern crate portaudio;
extern crate socool_parser;
extern crate weresocool;
use portaudio as pa;
use socool_parser::parser::*;
use weresocool::{
    write::write_composition_to_wav,
    generation::parsed_to_waveform::generate_composition,
    portaudio_setup::output::setup_portaudio_output,
    ui::{
        get_args,
        were_so_cool_logo,
        banner,
        no_file_name,
        printed
    },
};

fn main() -> Result<(), pa::Error> {
    were_so_cool_logo();
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
            Some(_filename) => {},
            _ => no_file_name()
    }

    let parsed = parse_file(&filename.unwrap().to_string());
    let main = parsed.table.get("main").unwrap();
    let init = parsed.init;
    let composition = generate_composition(init, main.clone());
    if args.is_present("print") {
        banner("Printing".to_string(), filename.unwrap().to_string());
        write_composition_to_wav(composition);
        printed()
    } else {
        banner("New Playing".to_string(), filename.unwrap().to_string());

        let pa = pa::PortAudio::new()?;
        let mut output_stream = setup_portaudio_output(composition, &pa)?;
        output_stream.start()?;

        while let true = output_stream.is_active()? {}

        output_stream.stop()?;
    }

    Ok(())
}
