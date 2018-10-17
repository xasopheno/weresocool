extern crate portaudio;
extern crate weresocool;
extern crate colored;
extern crate socool_parser;
use portaudio as pa;
use weresocool::{
    portaudio_setup::output::setup_portaudio_output,
    generation::parsed_to_waveform::generate_composition,
};
use std::env;
use colored::*;
use std::collections::HashMap;
use socool_parser::{
    parser::*,
    ast::*
};


fn main() -> Result<(), pa::Error> {
    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ".magenta().bold());
    println!("{}", "*** Make cool sounds. Impress your friends ***  ".cyan());
    println!("{}", " ~~~~“Catchy tunes for your next seizure.”~~~~".cyan());

    let args: Vec<String> = env::args().collect();
    let filename;
    if args.len() == 2 {
        filename = &args[1];
        println!("\n        Now Playing: {}\n", filename);
    } else {
        println!("\n{}\n", "Forgot to pass in a filename.".red().bold());
        println!("{}", "Example:".cyan());
        println!("{}\n", "./weresocool song.socool".cyan().italic());
        panic!("Wrong number of arguments.")

    }

    let parsed = parse_file(filename);
    let main = parsed.table.get("main").unwrap();
    let init = parsed.init;

    let pa = pa::PortAudio::new()?;
    let mut output_stream = setup_portaudio_output(generate_composition(init, main.clone()), &pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;

    Ok(())
}
