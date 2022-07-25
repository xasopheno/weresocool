use crate::Error;
use clap::ArgMatches;
use std::path::PathBuf;
use weresocool::generation::{RenderType, WavType};
use weresocool::interpretable::InputType;
use weresocool::interpretable::Interpretable;

pub fn print(print_args: Option<&ArgMatches>) -> Result<(), Error> {
    let args = print_args.ok_or_else(|| Error::Message("No print args".to_string()))?;

    let mut printed: Vec<&str> = vec![];
    let should_print = |target: &[&str]| -> bool {
        let result = target.iter().any(|arg| args.is_present(arg));
        result
    };
    let filename = args
        .values_of("file")
        .ok_or_else(|| {
            Error::Message(
                "Filename required. Usage: weresocool print [FILENAME] [FLAGS]".to_string(),
            )
        })?
        .collect::<Vec<_>>()
        .first()
        .expect("No Filename")
        .to_string();

    let mut output_dir = PathBuf::new();
    if let Some(values) = args.values_of("output_dir") {
        output_dir.push(values.collect::<Vec<_>>().first().expect("No Filename"));
    };

    println!("Filename: {}", filename);
    if should_print(&["all", "wav", "sound"]) {
        println!("printing .wav...");

        InputType::Filename(&filename).make(
            RenderType::Wav(WavType::Wav {
                cli: true,
                output_dir: output_dir.clone(),
            }),
            None,
        )?;
        printed.push("wav")
    }
    if should_print(&["all", "mp3", "sound"]) {
        println!("printing .mp3...");

        InputType::Filename(&filename).make(
            RenderType::Wav(WavType::Mp3 {
                cli: true,
                output_dir: output_dir.clone(),
            }),
            None,
        )?;
        printed.push("mp3")
    }
    if should_print(&["all", "oggvorbis", "sound"]) {
        println!("printing .ogg...");

        // InputType::Filename(&filename).make(
        // RenderType::Wav(WavType::OggVorbis {
        // cli: true,
        // output_dir: output_dir.clone(),
        // }),
        // None,
        // )?;
        // printed.push("ogg")
    }
    if should_print(&["all", "csv"]) {
        println!("printing .csv...");
        InputType::Filename(&filename).make(
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
        InputType::Filename(&filename).make(
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
        InputType::Filename(&filename).make(
            RenderType::Stems {
                cli: true,
                output_dir: output_dir.clone(),
            },
            None,
        )?;
        printed.push("stems")
    }
    if printed.is_empty() {
        InputType::Filename(&filename).make(
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
