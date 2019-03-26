extern crate portaudio;
extern crate rayon;
extern crate socool_ast;
extern crate socool_parser;
extern crate weresocool;
use portaudio as pa;
use socool_ast::ast::OpOrNf;
use socool_parser::parser::*;
use weresocool::{
    examples::documentation,
    generation::parsed_to_render::r_to_f64,
    generation::parsed_to_render::{render, to_json, to_wav},
    instrument::oscillator::Basis,
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
    let parsed_main = parsed.table.get("main").unwrap();

    let nf = match parsed_main {
        OpOrNf::Nf(nf) => nf,
        OpOrNf::Op(_) => panic!("main is Not in Normal Form for some terrible reason."),
    };

    let init = parsed.init;

    let basis = Basis {
        f: r_to_f64(init.f),
        g: r_to_f64(init.g),
        l: r_to_f64(init.l),
        p: r_to_f64(init.p),
        a: 44100.0,
        d: 44100.0,
    };

    if args.is_present("print") {
        let composition = render(&basis, &nf, &parsed.table);
        to_wav(composition, filename.unwrap().to_string());
    } else if args.is_present("json") {
        to_json(&basis, &nf, &parsed.table, filename.unwrap().to_string());
    } else {
        let composition = render(&basis, &nf, &parsed.table);

        let pa = pa::PortAudio::new()?;

        let mut output_stream = setup_portaudio_output(composition, &pa)?;

        banner("Now Playing".to_string(), filename.unwrap().to_string());

        output_stream.start()?;

        while let true = output_stream.is_active()? {}

        output_stream.stop()?;
    }

    Ok(())
}
