extern crate portaudio;
extern crate sound;
use sound::portaudio_setup::{setup_portaudio_input, setup_portaudio_output};
use sound::ring_buffer::RingBuffer;
use sound::settings::{get_default_app_settings, Settings};
use sound::sine;
use sound::yin::YinBuffer;
use std::sync::mpsc::channel;

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
    let (mut input_stream, input_callback_rx) = setup_portaudio_input(&pa, settings)?;

    let mut input_buffer: RingBuffer<f32> =
        RingBuffer::<f32>::new(settings.yin_buffer_size as usize);

    input_stream.start()?;
    let mut frequency: f32 = 0.0;

    while let true = input_stream.is_active()? {
        match input_callback_rx.recv() {
            Ok(vec) => {
                input_buffer.append(vec);
                let mut buffer_vec: Vec<f32> = input_buffer.to_vec();
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

    input_stream.stop()?;
    Ok(())
}
