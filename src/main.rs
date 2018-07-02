extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::analyze::Analyze;
use sound::input_output_setup::prepare_input;
use sound::oscillator::{Oscillator, R};
use sound::portaudio_setup::setup_portaudio_output;
use sound::settings::{get_default_app_settings, Settings};
use sound::state::{State, StateAPI};
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
        R::atio(13, 2, 0.0, 0.2),
        R::atio(13, 2, -11.0, 0.2),
        R::atio(11, 4, 0.0, 1.0),
        R::atio(11, 4, 0.0, 1.0),
        R::atio(9, 4, 0.0, 1.0),
        R::atio(9, 4, 6.0, 1.0),
        R::atio(7, 3, -3.0, 1.0),
        R::atio(1, 1, -3.0, 1.0),
        R::atio(5, 4, 0.0, 1.0),
        R::atio(11, 8, 0.0, 1.0),
        R::atio(1, 2, -3.0, 0.5),
        R::atio(1, 2, -0.0, 0.5),
    ];

    let l_ratios = vec![
        R::atio(15, 2, 18.0, 0.1),
        R::atio(15, 2, 0.0, 0.1),
        R::atio(10, 2, -9.0, 0.4),
        R::atio(7, 2, 1.0, 1.0),
        R::atio(7, 2, 0.0, 1.0),
        R::atio(12, 4, 0.0, 1.0),
        R::atio(15, 8, 0.0, 1.0),
        R::atio(15, 8, 6.0, 1.0),
        R::atio(3, 2, 3.0, 1.0),
        R::atio(1, 1, 0.0, 1.0),
        R::atio(1, 1, -2.0, 1.0),
        R::atio(1, 2, 0.0, 0.5),
        R::atio(1, 2, -2.0, 0.5),
    ];

    let settings: &'static Settings = get_default_app_settings();
    let pa = pa::PortAudio::new()?;

    let mut input = prepare_input(&pa, &settings)?;
    let oscillator = Oscillator::new(10, l_ratios, r_ratios);
    let oscillator_mutex: &mut Arc<Mutex<Oscillator>> = &mut Arc::new(Mutex::new(oscillator));

    let mut output_stream = setup_portaudio_output(&pa, &settings, Arc::clone(oscillator_mutex))?;

    input.stream.start()?;
    output_stream.start()?;

    while let true = input.stream.is_active()? {
        match input.callback_rx.recv() {
            Ok(vec) => {
                input.buffer.push_vec(vec);
                let mut osc = oscillator_mutex.lock().unwrap();
                let mut buffer_vec: Vec<f32> = input.buffer.to_vec();
                let gain = buffer_vec.gain();
                let (frequency, probability) =
                    buffer_vec.yin_pitch_detection(settings.sample_rate, settings.threshold);
//                println!("{}, {}", frequency, probability);

                osc.update(frequency, gain, probability);
            }
            _ => panic!(),
        }
    }

    input.stream.stop()?;
    output_stream.stop()?;
    Ok(())
}

// 500 : 1
//1000 : 0.7
//1500 : 0.49
//2000: 0.343
//2500: 0.240
