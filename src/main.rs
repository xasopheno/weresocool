extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::input_output_setup::{prepare_input, Oscillator, Output};
use sound::portaudio_setup::{get_output_settings, setup_portaudio_output};
use sound::settings::{get_default_app_settings, Settings};
use sound::sine::generate_sinewave;
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
    let settings: &'static Settings = get_default_app_settings();

    let pa = pa::PortAudio::new()?;
    let mut input = prepare_input(&pa, &settings)?;
    let mut frequency: &mut Arc<Mutex<isize>> = &mut Arc::new(Mutex::new(42));

    //    let sin_osc = &mut Oscillator {
    //        frequency: &mut frequency,
    //        phase: 0.0,
    //        generator: generate_sinewave,
    //    };
    let output_settings = get_output_settings(&pa, &settings);
    let mut output_stream = setup_portaudio_output(&pa, &settings, Arc::clone(frequency))?;
    //
    //    let mut output = Output {
    //        stream: output_stream,
    //        oscillator: sin_osc,
    //    };

    input.stream.start()?;
    output_stream.start()?;

    while let true = input.stream.is_active()? {
        match input.callback_rx.recv() {
            Ok(vec) => {
                let freq = *frequency.lock().unwrap();
                input.buffer.append(vec);
                let mut buffer_vec: Vec<f32> = input.buffer.to_vec();
                if buffer_vec.gain() > settings.gain_threshold {
                    *frequency.lock().unwrap() = buffer_vec
                        .yin_pitch_detection(settings.sample_rate, settings.threshold)
                        .floor() as isize;
                } else {
                    *frequency.lock().unwrap() = 0;
                }
            }
            _ => panic!(),
        }

        //        if freq > 0 && freq < 2500 {
        //            println!("******************{:?}", freq);
        //        }
    }

    input.stream.stop()?;
    output_stream.stop()?;
    Ok(())
}
