use colored::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn clear_screen() {
    if let Err(e) = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
    } else {
        std::process::Command::new("clear").status()
    } {
        eprintln!("Failed to clear the screen: {:?}", e);
    }
}

pub fn were_so_cool_logo(action: Option<&str>, filename: Option<String>) {
    clear_screen();
    println!(
        "{} {}",
        "\n**** WereSoCool".truecolor(250, 180, 220).bold(),
        format!("v{} ****", VERSION).truecolor(250, 180, 220).bold()
    );
    println!("{}", "--- Make cool sounds. ---".truecolor(250, 134, 200));
    println!(
        "{}",
        "--- Impress your friends/pets/plants. ---".truecolor(250, 134, 200)
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
