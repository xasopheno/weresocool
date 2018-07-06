extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::oscillator::{Oscillator};

use sound::portaudio_setup::setup_portaudio_duplex;
use sound::settings::get_default_app_settings;
use sound::ratios::{complicated_ratios, simple_ratios, R};

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

    let (l_ratios, r_ratios) = simple_ratios();
    let pa = pa::PortAudio::new()?;

    let oscillator = Oscillator::new(10, l_ratios, r_ratios, get_default_app_settings());
    let oscillator_mutex: &mut Arc<Mutex<Oscillator>> = &mut Arc::new(Mutex::new(oscillator));

    let mut duplex_stream = setup_portaudio_duplex(&pa, Arc::clone(oscillator_mutex))?;

    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
