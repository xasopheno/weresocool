extern crate portaudio;
extern crate socool_parser;
extern crate weresocool;
use portaudio as pa;
use socool_parser::parser::*;
use weresocool::{
    generation::parsed_to_waveform::{generate_composition, generate_events},
    portaudio_setup::output::setup_portaudio_output,
    ui::{banner, get_args, no_file_name, printed, were_so_cool_logo},
    write::{write_composition_to_json, write_composition_to_wav},
};

fn main() -> Result<(), pa::Error> {
    were_so_cool_logo();
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let parsed = parse_file(&filename.unwrap().to_string());
    let main = parsed.table.get("main").unwrap();
    let init = parsed.init;
    if args.is_present("print") {
        let composition = generate_composition(init, main.clone());
        banner("Printing".to_string(), filename.unwrap().to_string());
        write_composition_to_wav(composition);
        printed("WAV".to_string());
    } else if args.is_present("json") {
        banner("Printing".to_string(), filename.unwrap().to_string());
        let events = generate_events(init, main.clone());
        write_composition_to_json(events, &filename.unwrap().to_string())
            .expect("Writing to JSON failed");
        printed("JSON".to_string());
    } else {
        banner("New Playing".to_string(), filename.unwrap().to_string());
        let composition = generate_composition(init, main.clone());

        let pa = pa::PortAudio::new()?;
        let mut output_stream = setup_portaudio_output(composition, &pa)?;
        output_stream.start()?;

        while let true = output_stream.is_active()? {}

        output_stream.stop()?;
    }

    Ok(())
}
