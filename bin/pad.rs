//#![allow(dead_code, unused_imports, unused_variables)]
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use weresocool::{
    generation::parsed_to_render::sum_all_waveforms,
    instrument::StereoWaveform,
    interpretable::InputType::Filename,
    manager::{BufferManager, RenderManager},
    portaudio::real_time_managed_long,
    settings::{default_settings, Settings},
    ui::were_so_cool_logo,
};

use failure::Fail;
use weresocool_error::Error;

const SETTINGS: Settings = default_settings();

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

fn run() -> Result<(), Error> {
    were_so_cool_logo();
    println!("       )))***=== REAL<COOL>TIME *buffered ===***(((  \n ");

    let filename1 = "songs/dance/skip.socool";
    let filename2 = "songs/dance/candle.socool";

    let mut render_manager = Arc::new(Mutex::new(RenderManager::init_silent()));
    let mut render_manager_clone = render_manager.clone();

    let (send, recv) = channel();
    println!("Start...");
    thread::Builder::new()
        .name("Sender".to_string())
        .spawn(move || {
            thread::sleep(Duration::from_secs(1));
            for _ in 0..4 {
                send.send(Filename(&filename1)).unwrap();
                thread::sleep(Duration::from_secs(4));
                send.send(Filename(&filename2)).unwrap();
                thread::sleep(Duration::from_secs(4));
            }
        })?;

    thread::Builder::new()
        .name("Receiver".to_string())
        .spawn(move || loop {
            if let Ok(v) = recv.try_recv() {
                println!("language received");

                match render_manager_clone.lock().unwrap().prepare_render(v) {
                    Ok(_) => println!("Render Success"),
                    _ => panic!("Render Failure"),
                }
            };
        })?;

    let mut stream = real_time_managed_long(Arc::clone(&render_manager))?;
    stream.start()?;

    while let true = stream.is_active()? {}

    stream.stop()?;

    Ok(())
}
