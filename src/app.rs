use clap::{App, AppSettings, Arg, SubCommand};

pub fn app() -> clap::App<'static, 'static> {
    App::new("WereSoCool CLI")
        .version("1.0")
        .author("Danny Meyer")
        .about("Make cool sounds and impress your friends/pets/plants.")
        .setting(AppSettings::ColoredHelp)
        .subcommand(
            SubCommand::with_name("new")
                .help("new [filename.socool]")
                .arg(
                    Arg::with_name("file")
                        .multiple(false)
                        .number_of_values(1)
                        .index(1)
                        .help("filename"),
                ),
        )
        .subcommand(
            SubCommand::with_name("play")
                .help("play [filename.socool]")
                .arg(
                    Arg::with_name("file")
                        .multiple(false)
                        .number_of_values(1)
                        .index(1)
                        .help("filename"),
                ),
        )
        .subcommand(
            SubCommand::with_name("watch")
                .alias("dev")
                .help("dev [filename.socool]")
                .arg(
                    Arg::with_name("file")
                        .multiple(false)
                        .number_of_values(1)
                        .index(1)
                        .help("filename"),
                ),
        )
        .subcommand(
            SubCommand::with_name("print")
                .usage("weresocool print [filename.socool] [flags]")
                .arg(
                    Arg::with_name("output_dir")
                        .long("output_dir")
                        .value_name("OUTPUT_DIR")
                        .number_of_values(1)
                        .help("output_dir"),
                )
                .arg(
                    Arg::with_name("file")
                        .value_name("[filename.socool]")
                        .multiple(false)
                        .number_of_values(1)
                        .index(1)
                        .help("filename"),
                )
                .arg(
                    Arg::with_name("mp3")
                        .long("mp3")
                        .takes_value(false)
                        .help("print mp3 file"),
                )
                .arg(
                    Arg::with_name("oggvorbis")
                        .long("oggvorbis")
                        .takes_value(false)
                        .help("print oggvorbis file"),
                )
                .arg(
                    Arg::with_name("wav")
                        .long("wav")
                        .takes_value(false)
                        .help("print wav file"),
                )
                .arg(
                    Arg::with_name("csv")
                        .long("csv")
                        .takes_value(false)
                        .help("print csv file"),
                )
                .arg(
                    Arg::with_name("json")
                        .long("json")
                        .takes_value(false)
                        .help("print csv file"),
                )
                .arg(
                    Arg::with_name("stems")
                        .long("stems")
                        .takes_value(false)
                        .help("print stems as a zip file"),
                )
                .arg(
                    Arg::with_name("sound")
                        .long("sound")
                        .short("s")
                        .takes_value(false)
                        .help("print sound file"),
                )
                .arg(
                    Arg::with_name("all")
                        .long("all")
                        .short("a")
                        .takes_value(false)
                        .help("print all file types"),
                ),
        )
}
