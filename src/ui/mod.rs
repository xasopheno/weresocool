extern crate clap;
extern crate colored;
use self::clap::{App, Arg, ArgMatches};
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
        "*** Make cool sounds. Impress your friends/pets/plants ***  ".cyan()
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

pub fn printed(file_type: String) {
    println!(
        "{}{}{}",
        "\n ***** WereSoFinishedWritingThe".magenta().bold(),
        file_type.blue().bold(),
        "File ****** \n ".magenta().bold()
    );
}

pub fn get_args() -> ArgMatches<'static> {
    App::new("WereSoCool")
        .about("*** Make cool sounds. Impress your friends ***")
        .author("Danny Meyer <Danny.Meyer@gmail.com>")
        .arg(
            Arg::with_name("filename")
                .help("filename eg: my_song.socool")
                .required(false),
        )
        .arg(
            Arg::with_name("print")
                .help("Prints file to .wav")
                .short("p")
                .long("print"),
        )
        .arg(
            Arg::with_name("json")
                .help("Prints file to .json")
                .short("j")
                .long("json"),
        )
        .arg(
            Arg::with_name("doc")
                .help("Prints some documentation")
                .short("d")
                .long("doc"),
        )
        .get_matches()
}

pub fn get_test_args() -> ArgMatches<'static> {
    App::new("WereSoCoolTest")
        .about("*** Make cool tests. Impress your friends ***")
        .author("Danny Meyer <Danny.Meyer@gmail.com>")
        .arg(
            Arg::with_name("rehash")
                .help("Recalculate Hashes")
                .short("r")
                .long("rehash"),
        )
        .get_matches()
}
