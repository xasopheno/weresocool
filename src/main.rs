extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::fader::Fader;
use sound::input_output_setup::prepare_input;
use sound::oscillator::{Oscillator, R};
use sound::portaudio_setup::setup_portaudio_output;
use sound::settings::{get_default_app_settings, Settings};
use sound::yin::YinBuffer;
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
    println!("{}", "\n ***** Rust DSP ****** \n ");
    let settings: &'static Settings = get_default_app_settings();
    let pa = pa::PortAudio::new()?;

    let ratios = vec![
        R::atio(8, 1),
        R::atio(15, 4),
        R::atio(11, 4),
        R::atio(7, 1),
        R::atio(7, 3),
        R::atio(5, 2),
        R::atio(6, 1),
        R::atio(4, 1),
        R::atio(2, 1),
        R::atio(3, 2),
        R::atio(6, 5),
        R::atio(1, 1),
        R::atio(1, 1),
        R::atio(1, 2),
        R::atio(1, 4),
    ];

    let fader = Fader::new(256, 500, settings.output_buffer_size as usize);

    let mut input = prepare_input(&pa, &settings)?;
    let oscillator = Oscillator::new(10, ratios, fader);
    let oscillator_mutex: &mut Arc<Mutex<Oscillator>> = &mut Arc::new(Mutex::new(oscillator));

    let mut output_stream = setup_portaudio_output(&pa, &settings, Arc::clone(oscillator_mutex))?;

    input.stream.start()?;
    output_stream.start()?;

    while let true = input.stream.is_active()? {
        match input.callback_rx.recv() {
            Ok(vec) => {
                input.buffer.push_vec(vec);
                let mut osc = oscillator_mutex.lock().unwrap();
                // println!("{:?}", osc.f_buffer.current());
                let mut buffer_vec: Vec<f32> = input.buffer.to_vec();
                let gain = buffer_vec.gain();
                let freq = buffer_vec
                    .yin_pitch_detection(settings.sample_rate, settings.threshold)
                    .floor();
                osc.update(freq, gain);
            }
            _ => panic!(),
        }
    }

    input.stream.stop()?;
    output_stream.stop()?;
    Ok(())
}
