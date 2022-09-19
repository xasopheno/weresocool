mod app;
mod demo;
mod new;
mod play;
mod print;
mod test;
mod watch;
use crate::play::{
    play,
    Play::{Once, Watch},
};
use notify::Error as NotifyError;
#[cfg(feature = "app")]
use portaudio::error::Error as PortAudioError;
use std::env;
use thiserror::Error;
use weresocool::error::Error as WscError;
use weresocool::ui::were_so_cool_logo;

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
    were_so_cool_logo();
    let cwd = env::current_dir()?;

    let matches = app::app().get_matches();

    match matches.subcommand() {
        ("new", new_args) => new::new(new_args, cwd)?,
        ("demo", _) => demo::demo()?,
        ("play", play_args) => play(play_args, cwd, Once)?,
        ("watch", play_args) => play(play_args, cwd, Watch)?,
        ("print", print_args) => print::print(print_args)?,
        _e => {
            app::app().print_help().unwrap();
        }
    }
    Ok(())
}
