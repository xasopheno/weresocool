extern crate colored;
extern crate clap;
use self::clap::{Arg, App, ArgMatches};
use colored::*;


pub fn were_so_cool_logo() {
    println!(
        "{}",
        "\n  ****** WereSoCool __!Now In Stereo!__ ****** "
            .magenta()
            .bold()
    );
    println!(
        "{}",
        "*** Make cool sounds. Impress your friends ***  ".cyan()
    );
    println!(
        "{}",
        " ~~~~“Catchy tunes for your next seizure.”~~~~".cyan()
    );
}

pub fn banner(name: String, s: String) {
    println!("\n        {}: {}\n", name, s);
}

pub fn no_file_name() {
    println!("\n{}\n", "Forgot to pass in a filename.".red().bold());
    println!("{}", "Example:".cyan());
    println!("{}\n", "./wsc song.socool".cyan().italic());
    panic!("Wrong number of arguments.")
}

pub fn printed() {
    println!("{}", "\n ***** WereSoFinishedWritingTheWavFile ****** \n ".magenta().bold());
}

pub fn get_args() -> ArgMatches<'static> {
    App::new("Were So Cool")
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
        .get_matches()
}
