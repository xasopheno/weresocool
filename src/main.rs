mod app;
mod demo;
mod fmt;
mod new;
mod play;
mod print;
mod test;
mod watch;

use crate::play::Play::{Once, Watch};
use colored::*;
use notify::Error as NotifyError;
use std::env;
use std::process::ExitCode;
use thiserror::Error;
use weresocool::error::Error as WscError;
use weresocool_portaudio::error::Error as PortAudioError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    WereSoCoolError(#[from] WscError),
    #[cfg(feature = "app")]
    #[error("{0}")]
    PortAudioError(#[from] PortAudioError),
    #[error("File watch error: {0}")]
    NotifyError(#[from] NotifyError),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\n{}\n", e.to_string().red());
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Error> {
    let cwd = env::current_dir()?;

    let matches = app::app().get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            new::new(sub_matches.get_one::<String>("filename").unwrap(), cwd)?
        }
        Some(("play", sub_matches)) => {
            let play_type = if sub_matches.get_flag("watch") {
                Watch
            } else {
                Once
            };
            #[cfg(target_os = "macos")]
            if sub_matches.get_flag("midi") {
                // Best-effort spawn of weresocool_midi UDP bridge (direct, so stdout is visible)
                let server_path = std::env::var("WSC_MIDI_SERVER")
                    .unwrap_or_else(|_| "/Users/danny/code/weresocool_midi/target/debug/rust-midi2-ump-jit".to_string());
                match std::process::Command::new(&server_path).spawn() {
                    Ok(_) => {}
                    Err(e) => eprintln!("Failed to start MIDI server at '{}': {}", server_path, e),
                }
            }
            let quiet = sub_matches.get_flag("quiet");
            play::play(
                sub_matches.get_one::<String>("filename").unwrap(),
                cwd,
                play_type,
                quiet,
            )?;
        }
        Some(("watch", sub_matches)) => {
            #[cfg(target_os = "macos")]
            if sub_matches.get_flag("midi") {
                let server_path = std::env::var("WSC_MIDI_SERVER")
                    .unwrap_or_else(|_| "/Users/danny/code/weresocool_midi/target/debug/rust-midi2-ump-jit".to_string());
                match std::process::Command::new(&server_path).spawn() {
                    Ok(_) => {}
                    Err(e) => eprintln!("Failed to start MIDI server at '{}': {}", server_path, e),
                }
            }
            let quiet = sub_matches.get_flag("quiet");
            play::play(
                sub_matches.get_one::<String>("filename").unwrap(),
                cwd,
                Watch,
                quiet,
            )?
        }
        Some(("demo", _)) => demo::demo()?,
        Some(("fmt", sub_matches)) => fmt::fmt(sub_matches)?,
        Some(("print", sub_matches)) => print::print(sub_matches)?,
        _e => {
            app::app().print_help().unwrap();
        }
    }
    Ok(())
}
