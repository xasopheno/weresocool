extern crate sound;
extern crate portaudio;
use std::sync::mpsc::channel;

use sound::ring_buffer::RingBuffer;
use sound::yin::YinBuffer;
use sound::{set_elements, sine};

use portaudio as pa;

const SAMPLE_RATE: f32 = 44_100.0;
const BUFFER_SIZE: f32 = 2048.0;
const CHUNK_SIZE: usize = 512;
const THRESHOLD: f32 = 0.20;
const CHANNELS: i32 = 1;
const INTERLEAVED: bool = true;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    let (mut stream, rx) = setup()?;
    let mut buffer: RingBuffer<f32> = RingBuffer::<f32>::new(BUFFER_SIZE as usize);
    stream.start()?;

    while let true = stream.is_active()? {
        match rx.recv() {
                Ok(vec) => {
                    buffer.append(vec);
                    let mut buffer_vec: Vec<f32> = buffer.to_vec();
                    if buffer_vec.gain() > -100.0 {
                        println!(
                            "{:?}",
                            buffer_vec.yin_pitch_detection(SAMPLE_RATE, THRESHOLD).floor()
                        );
                    }
                }
                _ => panic!(),
        }
    }

    stream.stop()?;
    Ok(())
}

fn setup() -> Result<
    (
        portaudio::Stream<portaudio::NonBlocking, portaudio::Input<f32>>,
        std::sync::mpsc::Receiver<Vec<f32>>,
    ),
    pa::Error,
> {

    let pa = pa::PortAudio::new()?;

    let (input_params, output_params) = setup_params(&pa)?;

    let (tx, rx) = channel();

    let settings =
        pa::InputStreamSettings::new(input_params, SAMPLE_RATE as f64, CHUNK_SIZE as u32);
    let stream = pa.open_non_blocking_stream(settings, move |args| {
        tx.send(args.buffer.to_vec()).unwrap();
        pa::Continue
    })?;

Ok((stream, rx))
}

fn setup_params (ref pa: &pa::PortAudio) -> Result<(
    pa::stream::Parameters<f32>,
    pa::stream::Parameters<f32>
    ), pa::Error> 
{
    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    println!("Default input device info: {:#?}", &input_info);

    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    println!("Default output device info: {:#?}", &output_info);

    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    let latency = output_info.default_low_output_latency;
    let output_params = pa::StreamParameters::new(def_output, CHANNELS, INTERLEAVED, latency);

    Ok((input_params, output_params))
}
