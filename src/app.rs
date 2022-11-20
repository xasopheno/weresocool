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
                .about("Render a .socool file")
                .arg(arg!([filename]).required(true)),
        )
        .subcommand(
            Command::new("watch")
                .about("Watch a .socool file. On file save, the composition will be re-rendered")
                .arg(arg!([filename]).required(true)),
        )
        .subcommand(Command::new("demo").about("Hear a cool sound"))
        .subcommand(
            Command::new("print")
                .about("Print a .socool composition to a file")
                .arg(arg!([filename]).required(false))
                .arg(arg!([output_dir]).required(false))
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
