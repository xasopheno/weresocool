extern crate portaudio;
extern crate sound;
use sound::input_output_setup::{ prepare_input, prepare_output };
use sound::settings::{get_default_app_settings, Settings};
use sound::sine;
use sound::yin::YinBuffer;

use portaudio as pa;

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
    let mut output = prepare_output( &pa, &settings)?;

    input.stream.start()?;
    output.stream.start()?;
    let mut frequency: f32 = 0.0;

    while let true = input.stream.is_active()? {
        match input.callback_rx.recv() {
            Ok(vec) => {
                input.buffer.append(vec);
                let mut buffer_vec: Vec<f32> = input.buffer.to_vec();
                if buffer_vec.gain() > settings.gain_threshold {
                    frequency = buffer_vec
                        .yin_pitch_detection(settings.sample_rate, settings.threshold)
                        .floor();
                }
            }
            _ => panic!(),
        }

        if frequency > 0.0 && frequency < 2000.0 {
            println!("{:?}", frequency);
        }
    }

    input.stream.stop()?;
    output.stream.stop()?;
    Ok(())
}
