use crate::Error;
use clap::ArgMatches;
use std::path::PathBuf;
use weresocool::generation::{RenderType, WavType};
use weresocool::interpretable::InputType;
use weresocool::interpretable::Interpretable;

pub fn print(print_args: &ArgMatches) -> Result<(), Error> {
    let mut printed: Vec<&str> = vec![];
    let should_print = |target: &[&str]| -> bool {
        let result = target.iter().any(|arg| print_args.get_flag(arg));
        result
    };
    let filename = print_args.get_one::<String>("filename").unwrap();

    let mut output_dir = PathBuf::new();
    if let Some(value) = print_args.get_one::<String>("output_dir") {
        output_dir.push(value);
    };

    println!("Filename: {}", filename);
    if should_print(&["all", "wav", "sound"]) {
        println!("printing .wav...");

        InputType::Filename(filename).make(
            RenderType::Wav(WavType::Wav {
                cli: true,
                output_dir: output_dir.clone(),
            }),
            None,
        )?;
        printed.push("wav")
    }

    #[cfg(not(target_os = "windows"))]
    if should_print(&["all", "mp3", "sound"]) {
        println!("printing .mp3...");

        InputType::Filename(filename).make(
            RenderType::Wav(WavType::Mp3 {
                cli: true,
                output_dir: output_dir.clone(),
            }),
            None,
        )?;
        printed.push("mp3")
    }

    #[cfg(target_os = "windows")]
    if should_print(&["all", "mp3", "sound"]) {
        println!("Printing to mp3 not yet supported on windows");
        printed.push("mp3");
    }

    #[cfg(not(target_os = "windows"))]
    if should_print(&["all", "oggvorbis", "sound"]) {
        println!("printing .ogg...");

        InputType::Filename(filename).make(
            RenderType::Wav(WavType::OggVorbis {
                cli: true,
                output_dir: output_dir.clone(),
            }),
            None,
        )?;
        printed.push("ogg")
    }

    #[cfg(target_os = "windows")]
    if should_print(&["all", "oggvorbis", "sound"]) {
        println!("Printing to oggvorbis not yet supported on windows");
        printed.push("ogg");
    }

    if should_print(&["all", "csv"]) {
        println!("printing .csv...");
        InputType::Filename(filename).make(
            RenderType::Csv1d {
                cli: true,
                output_dir: output_dir.clone(),
            },
            None,
        )?;
        printed.push("csv")
    }
    if should_print(&["all", "json"]) {
        println!("printing .json...");
        InputType::Filename(filename).make(
            RenderType::Json4d {
                cli: true,
                output_dir: output_dir.clone(),
            },
            None,
        )?;
        printed.push("json")
    }
    if should_print(&["all", "stems"]) {
        println!("printing .stems...");
        InputType::Filename(filename).make(
            RenderType::Stems {
                cli: true,
                output_dir: output_dir.clone(),
            },
            None,
        )?;
        printed.push("stems")
    }
    if printed.is_empty() {
        InputType::Filename(filename).make(
            RenderType::Wav(WavType::Wav {
                cli: true,
                output_dir,
            }),
            None,
        )?;
        println!("printing .wav (default)...");
    }

    println!("\tdone");
    Ok(())
}
