use crate::watch::watch;
use crate::Error;
use clap::ArgMatches;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use weresocool::interpretable::InputType::Filename;
use weresocool::manager::prepare_render_outside;
use weresocool::manager::RenderManager;
use weresocool::portaudio::real_time_render_manager;
use weresocool_instrument::RenderVoice;

pub enum Play {
    Once,
    Watch,
}

pub fn play(play_args: Option<&ArgMatches>, cwd: PathBuf, play: Play) -> Result<(), Error> {
    let filename = play_args
        .ok_or_else(|| Error::Message("No play args".to_string()))?
        .values_of("file")
        .ok_or_else(|| Error::Message("No value of file".to_string()))?
        .collect::<Vec<_>>()
        .first()
        .expect("No filename")
        .to_string();
    play_file(filename, cwd, play)?;
    Ok(())
}

pub fn play_file(filename: String, working_path: PathBuf, play: Play) -> Result<(), Error> {
    let render_voices = prepare_render_outside(Filename(&filename), Some(working_path.clone()));

    match play {
        Play::Once => play_once(render_voices?),
        Play::Watch => play_watch(filename, working_path, render_voices?),
    }
}

pub fn play_once(render_voices: Vec<RenderVoice>) -> Result<(), Error> {
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    let render_manager = Arc::new(Mutex::new(RenderManager::init(
        render_voices,
        None,
        // Option<KillChannel>
        Some(tx),
        // play once
        true,
    )));
    let mut stream = real_time_render_manager(Arc::clone(&render_manager))?;

    stream.start()?;
    // rx.recv blocks until it receives data and
    // after that, the function will complete,
    // stream will be dropped, and the application
    // will exit.
    match rx.recv() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
    Ok(())
}

fn play_watch(
    filename: String,
    working_path: PathBuf,
    render_voices: Vec<RenderVoice>,
) -> Result<(), Error> {
    let render_manager = Arc::new(Mutex::new(RenderManager::init(
        render_voices,
        None,
        None,
        false,
    )));
    watch(filename, working_path, render_manager.clone())?;
    let mut stream = real_time_render_manager(Arc::clone(&render_manager))?;
    stream.start()?;
    std::thread::park();
    Ok(())
}
