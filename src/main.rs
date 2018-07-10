extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::oscillator::{Oscillator};

use weresocool::portaudio_setup::setup_portaudio_duplex;
use weresocool::settings::get_default_app_settings;
use std::sync::{Arc, Mutex};

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!("{}", "\n ***** WereSoCool __!Now In Stereo!__ ****** \n ");

    let pa = pa::PortAudio::new()?;
    let mut duplex_stream = setup_portaudio_duplex(&pa)?;

    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
