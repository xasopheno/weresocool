mod app;
mod demo;
mod new;
mod play;
mod print;
mod test;
mod watch;

use crate::play::Play::{Once, Watch};
use notify::Error as NotifyError;
use std::env;
use thiserror::Error;
use weresocool::error::Error as WscError;
#[cfg(feature = "app")]
use weresocool_portaudio::error::Error as PortAudioError;
use weresocool_shared::Settings;

#[derive(Error, Debug)]
pub enum Error {
    #[error("WereSoCoolError: `{0}`")]
    WereSoCoolError(#[from] WscError),
    #[cfg(feature = "app")]
    #[error("PortAudioError: `{0}`")]
    PortAudioError(#[from] PortAudioError),
    #[error("NotifyError: `{0}`")]
    NotifyError(#[from] NotifyError),
    #[error("IoError: `{0}`")]
    IoError(#[from] std::io::Error),
    #[error("`{0}")]
    Message(String),
}

fn main() -> Result<(), Error> {
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
            play::play(
                sub_matches.get_one::<String>("filename").unwrap(),
                cwd,
                play_type,
            )?;
        }
        Some(("watch", sub_matches)) => play::play(
            sub_matches.get_one::<String>("filename").unwrap(),
            cwd,
            Watch,
        )?,
        Some(("demo", _)) => demo::demo()?,
        Some(("print", sub_matches)) => print::print(sub_matches)?,
        _e => {
            app::app().print_help().unwrap();
        }
    }
    Ok(())
}
