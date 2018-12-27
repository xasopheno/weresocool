extern crate portaudio;
extern crate rayon;
extern crate socool_parser;
extern crate weresocool;
use portaudio as pa;
use socool_parser::parser::*;
use weresocool::{
    examples::documentation,
    generation::parsed_to_render::{render, to_json, to_wav},
    portaudio_setup::output::setup_portaudio_output,
    ui::{banner, get_args, no_file_name, were_so_cool_logo},
};

fn main() -> Result<(), pa::Error> {
    were_so_cool_logo();
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
    let init = parsed.init;

    for (name, op) in parsed.table.clone()  {
        println!("{:?}", name);
    }

    if args.is_present("print") {
        let composition = render(main, init);
        to_wav(composition, filename.unwrap().to_string());
    } else if args.is_present("json") {
        to_json(main, init, filename.unwrap().to_string());
    } else {
        let composition = render(main, init);

        let pa = pa::PortAudio::new()?;

        let mut output_stream = setup_portaudio_output(composition, &pa)?;

        banner("Now Playing".to_string(), filename.unwrap().to_string());

        output_stream.start()?;

        while let true = output_stream.is_active()? {}

        output_stream.stop()?;
    }

    Ok(())
}
