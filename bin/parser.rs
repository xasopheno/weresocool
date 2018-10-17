extern crate colored;
extern crate portaudio;
extern crate socool_parser;
extern crate weresocool;
extern crate clap;
use colored::*;
use portaudio as pa;
use socool_parser::parser::*;
use weresocool::{
    generation::parsed_to_waveform::generate_composition,
    portaudio_setup::output::setup_portaudio_output,
    ui::were_so_cool_logo,
};

use clap::{Arg, App, SubCommand};

fn main() -> Result<(), pa::Error> {
    were_so_cool_logo();

    let matches = App::new("Were So Cool")
        .about("*** Make cool sounds. Impress your friends ***")
        .author("Danny Meyer <Danny.Meyer@gmail.com>")
        .arg(Arg::with_name("filename")
                 .help("filename eg: my_song.socool")
                 .required(false)
        )
        .arg(Arg::with_name("print")
            .help("Prints file to .wav")
            .short("p")
            .long("print"))
        .get_matches();
    
        let filename = matches.value_of("filename");
        match filename {
                Some(_filename) => {},
                _ => {        println!("\n{}\n", "Forgot to pass in a filename.".red().bold());
                println!("{}", "Example:".cyan());
                println!("{}\n", "./weresocool song.socool".cyan().italic());
                panic!("Wrong number of arguments.")
            }

        }

        let parsed = parse_file(&filename.unwrap().to_string());
        let main = parsed.table.get("main").unwrap();
        let init = parsed.init;

    if matches.is_present("print") {
        println!("\n        Printing: {}\n", &filename.unwrap().to_string());
        println!("{}", "This would print instead of run");
    } else {
        println!("\n        Now Playing: {}\n", &filename.unwrap().to_string());
        let pa = pa::PortAudio::new()?;
        let mut output_stream = setup_portaudio_output(generate_composition(init, main.clone()), &pa)?;
        output_stream.start()?;

        while let true = output_stream.is_active()? {}

        output_stream.stop()?;
    }

    Ok(())
}
