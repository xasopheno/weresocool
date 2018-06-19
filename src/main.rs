extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::input_output_setup::prepare_input;
use sound::portaudio_setup::setup_portaudio_output;
use sound::settings::{get_default_app_settings, Settings};
use sound::sine::{generate_sinewave};
use sound::oscillator::{Oscillator};
use sound::yin::YinBuffer;
use sound::ring_buffer::RingBuffer;
use sound::fader::{Fader};
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
    let settings: &'static Settings = get_default_app_settings();
    let pa = pa::PortAudio::new()?;

    let mut input = prepare_input(&pa, &settings)?;
    let oscillator: &mut Arc<Mutex<Oscillator>> = &mut Arc::new(Mutex::new(Oscillator {
        f_buffer: RingBuffer::<f32>::new_full(10 as usize),
        phase: (0.0, 0.0, 0.0),
        generator: generate_sinewave,
        fader: Fader::new(256, settings.output_buffer_size as usize),
        faded_in: false,
    }));

    let mut output_stream =
        setup_portaudio_output(&pa, &settings, Arc::clone(oscillator))?;

    input.stream.start()?;
    output_stream.start()?;

    while let true = input.stream.is_active()? {
        match input.callback_rx.recv() {
            Ok(vec) => {
                input.buffer.push_vec(vec);
                let mut osc = oscillator.lock().unwrap();
                println!("{:?}", osc.f_buffer.current());
                let mut buffer_vec: Vec<f32> = input.buffer.to_vec();
                if buffer_vec.gain() > settings.gain_threshold {
                    let freq =buffer_vec
                        .yin_pitch_detection(settings.sample_rate, settings.threshold)
                        .floor();
                    if freq < 2500.0 {
                        osc.f_buffer.push(freq);
                    }
                } else {
                    osc.f_buffer.push(0.0);
                }
            }
            _ => panic!(),
        }
    }

    input.stream.stop()?;
    output_stream.stop()?;
    Ok(())
}
