use clap::{App, Arg, ArgMatches};
use colored::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn clear_screen() {
    if cfg!(unix) {
        std::process::Command::new("clear").status().unwrap();
    } else if cfg!(windows) {
        std::process::Command::new("cls").status().unwrap();
    }
}

pub fn were_so_cool_logo(action: Option<&str>, filename: Option<String>) {
    clear_screen();
    println!(
        "{} {}",
        "\n**** WereSoCool".truecolor(250, 180, 220).bold(),
        format!("v{} ****", VERSION).truecolor(250, 180, 220).bold()
    );
    println!(
        "{}",
        "--- Make cool sounds. Impress your friends/pets/plants. ---".truecolor(250, 134, 200)
    );
    // println!(
    // "{}",
    // "~~~ Catchy tunes for your next seizure. ~~~"
    // .truecolor(10, 180, 250)
    // .italic(),
    // );

    if let Some(a) = action {
        if let Some(f) = filename {
            println!("{}", format!("~> {}: {} <~", a, f).truecolor(10, 180, 250));
        }
    }
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
            Arg::with_name("csv")
                .help("Prints file to .csv")
                .short("c")
                .long("csv"),
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
