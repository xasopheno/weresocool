//#![allow(dead_code, unused_imports, unused_variables)]
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use weresocool::{
    generation::parsed_to_render::sum_all_waveforms,
    instrument::StereoWaveform,
    interpretable::InputType::Filename,
    portaudio::real_time_managed::real_time_managed,
    render_manager::{BufferManager, RenderManager},
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

    let mut render_manager = RenderManager::init_silent();
    let buffer_manager = Arc::new(Mutex::new(BufferManager::init_silent()));
    let buffer_manager_clone = Arc::clone(&buffer_manager);

    let (send, recv) = channel();
    println!("Start...");
    thread::Builder::new()
        .name("Sender".to_string())
        .spawn(move || {
            for _ in 0..4 {
                send.send(Filename(&filename1)).unwrap();
                thread::sleep(Duration::from_secs(16));
                send.send(Filename(&filename2)).unwrap();
                thread::sleep(Duration::from_secs(20));
            }
        })?;

    thread::Builder::new()
        .name("Receiver".to_string())
        .spawn(move || loop {
            if let Ok(v) = recv.try_recv() {
                println!("new language received");

                match render_manager.prepare_render(v) {
                    Ok(_) => buffer_manager_clone
                        .lock()
                        .unwrap()
                        .inc_render_write_buffer(),
                    _ => panic!("Need to handle failed preparation"),
                }
            };

            let batch: Option<Vec<StereoWaveform>> =
                render_manager.render_batch(SETTINGS.buffer_size);

            if let Some(b) = batch {
                if !b.is_empty() {
                    let stereo_waveform = sum_all_waveforms(b);
                    buffer_manager_clone.lock().unwrap().write(stereo_waveform);
                }
            }
        })?;

    let mut stream = real_time_managed(Arc::clone(&buffer_manager))?;
    stream.start()?;

    while let true = stream.is_active()? {}

    stream.stop()?;

    Ok(())
}
