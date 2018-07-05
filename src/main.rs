extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::oscillator::{Oscillator, R};
use sound::portaudio_setup::setup_portaudio_duplex;
use sound::settings::get_default_app_settings;
//use sound::state::{State, StateAPI};
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
    println!("{}", "\n ***** Rust DSP __!Now In Stereo!__ ****** \n ");

    let r_ratios = vec![
        R::atio(21, 2, 0.0, 0.05),
        R::atio(21, 2, 0.2, 0.05),
        R::atio(17, 2, 0.0, 0.1),
        R::atio(17, 2, 0.3, 0.1),
        R::atio(13, 2, 0.0, 0.15),
        R::atio(13, 2, -11.0, 0.15),
        R::atio(11, 2, 5.0, 0.15),
        R::atio(11, 2, 0.0, 0.15),
        R::atio(12, 4, 0.0, 1.0),
        R::atio(9, 4, 0.0, 1.0),
        R::atio(9, 4, 6.0, 1.0),
        R::atio(5, 4, 0.0, 1.0),
        R::atio(7, 3, -3.0, 1.0),
        R::atio(11, 8, 0.0, 1.0),
        R::atio(1, 1, -3.0, 1.0),
        R::atio(1, 2, -0.0, 0.5),
        R::atio(1, 2, 0.5, 0.5),
        R::atio(1, 4, 1.25, 0.6),
        R::atio(1, 4, 0.0, 0.6),
    ];

    let l_ratios = vec![
        R::atio(23, 2, 0.0, 0.04),
        R::atio(23, 2, -0.1, 0.04),
        R::atio(19, 2, 0.0, 0.1),
        R::atio(19, 2, -0.2, 0.1),
        R::atio(15, 2, 18.0, 0.15),
        R::atio(15, 2, 0.0, 0.15),
        R::atio(10, 2, -9.0, 0.15),
        R::atio(7, 2, 1.0, 1.0),
        R::atio(7, 2, 0.0, 1.0),
        R::atio(3, 2, 3.0, 1.0),
        R::atio(12, 4, 0.0, 1.0),
        R::atio(15, 8, 0.0, 1.0),
        R::atio(15, 8, 6.0, 1.0),
        R::atio(1, 1, 0.0, 1.0),
        R::atio(1, 1, -2.0, 1.0),
        R::atio(1, 2, 0.0, 0.5),
        R::atio(1, 2, 0.5, 0.5),
        R::atio(1, 4, 1.0, 0.6),
        R::atio(1, 4, 0.0, 0.6),
    ];

    let pa = pa::PortAudio::new()?;

    let oscillator = Oscillator::new(10, l_ratios, r_ratios, get_default_app_settings());
    let oscillator_mutex: &mut Arc<Mutex<Oscillator>> = &mut Arc::new(Mutex::new(oscillator));

    let mut duplex_stream = setup_portaudio_duplex(&pa, Arc::clone(oscillator_mutex))?;

    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}

// 500 : 1
//1000 : 0.7
//1500 : 0.49
//2000: 0.343
//2500: 0.240
