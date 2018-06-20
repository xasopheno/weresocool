extern crate portaudio;
extern crate sound;
use portaudio as pa;
use sound::input_output_setup::prepare_input;
use sound::portaudio_setup::setup_portaudio_output;
use sound::settings::{get_default_app_settings, Settings};
use sound::sine::{generate_waveform};
use sound::oscillator::{Oscillator, R};
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
        phases: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ratios: vec![
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
        ],
        generator: generate_waveform,
        fader: Fader::new(256, 500, settings.output_buffer_size as usize),
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
                // println!("{:?}", osc.f_buffer.current());
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
