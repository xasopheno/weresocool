use clap::{arg, Arg, ArgAction, Command};
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn app() -> clap::Command {
    Command::new("WereSoCool CLI")
        .version(VERSION)
        .author("Danny Meyer")
        .about("Make cool sounds and impress your friends/pets/plants.")
        .subcommand(
            Command::new("new")
                .about("Create a new .socool file from the template")
                .arg(arg!([filename]).required(true)),
        )
        .subcommand(
            Command::new("play")
                .about("Render a .socool file. Run with --watch to rerender on file save")
                .arg(arg!([filename]).required(true))
                .arg(
                    Arg::new("watch")
                        .long("watch")
                        .action(ArgAction::SetTrue)
                        .help("On file save, the composition will be re-rendered"),
                )
                .arg(
                    Arg::new("midi")
                        .long("midi")
                        .action(ArgAction::SetTrue)
                        .help("Enable MIDI output bridge and auto-run weresocool_midi server"),
                ),
        )
        .subcommand(
            Command::new("watch")
                .about("Same as play --watch")
                .arg(arg!([filename]).required(true))
                .arg(
                    Arg::new("midi")
                        .long("midi")
                        .action(ArgAction::SetTrue)
                        .help("Enable MIDI output bridge and auto-run weresocool_midi server"),
                ),
        )
        .subcommand(Command::new("demo").about("Hear a cool sound"))
        .subcommand(
            Command::new("fmt")
                .about("Format a .socool file")
                .arg(arg!([filename]).required(true))
                .arg(
                    Arg::new("inplace")
                        .short('i')
                        .long("inplace")
                        .action(ArgAction::SetTrue)
                        .help("Edit file in place"),
                )
                .arg(
                    Arg::new("check")
                        .long("check")
                        .action(ArgAction::SetTrue)
                        .help("Check if file is formatted (exit 1 if not)"),
                )
                .arg(
                    Arg::new("stdin")
                        .long("stdin")
                        .action(ArgAction::SetTrue)
                        .help("Read from stdin instead of file"),
                ),
        )
        .subcommand(
            Command::new("print")
                .about("Print a .socool composition to a file")
                .arg(arg!([filename]).required(false))
                .arg(arg!(--"output_dir" <output_dir>).required(false))
                .arg(
                    Arg::new("wav")
                        .long("wav")
                        .action(ArgAction::SetTrue)
                        .help("print a wav file (default)"),
                )
                .arg(
                    Arg::new("mp3")
                        .long("mp3")
                        .action(ArgAction::SetTrue)
                        .help("print an mp3"),
                )
                .arg(
                    Arg::new("oggvorbis")
                        .long("oggvorbis")
                        .action(ArgAction::SetTrue)
                        .help("print an oggvorbis file"),
                )
                .arg(
                    Arg::new("csv")
                        .long("csv")
                        .action(ArgAction::SetTrue)
                        .help("print a csv file"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("print a json file"),
                )
                .arg(
                    Arg::new("stems")
                        .long("stems")
                        .action(ArgAction::SetTrue)
                        .help("print stems as a zip file"),
                )
                .arg(
                    Arg::new("sound")
                        .long("sound")
                        .action(ArgAction::SetTrue)
                        .help("print all sound file types"),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .action(ArgAction::SetTrue)
                        .help("print all file types"),
                ),
        )
}
