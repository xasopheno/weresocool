extern crate sound;
extern crate portaudio;

use sound::{ sine, set_elements };
use sound::yin::{ self, YinBuffer };

use portaudio as pa;

const SAMPLE_RATE: f32 = 44_100.0;
const BUFFER_SIZE: f32 = 2048.0;
const THRESHOLD: f32 = 0.20;
const CHANNELS: i32 = 1;
const INTERLEAVED: bool = true;

fn main() { 
    let freq: f32 = 440.0;
    println!("generated freq is {}", freq);
    let mut buffer = sine::generate_sinewave(SAMPLE_RATE, BUFFER_SIZE, freq);
    println!("{}", buffer.yin_pitch_detection(SAMPLE_RATE, THRESHOLD));

    match run() {
        Ok(_) => {},
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    let mut stream = setup()?;
    stream.start()?;

    while let true = stream.is_active()? {}
    
    stream.stop()?;
    Ok(())
}

fn setup() -> Result<portaudio::Stream<portaudio::NonBlocking, portaudio::Input<f32>>, pa::Error> {
    let pa = pa::PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    println!("Default input device info: {:#?}", &input_info);

    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE as f64, BUFFER_SIZE as u32);

    let callback = move |pa::InputStreamCallbackArgs { buffer, .. }| {
            println!("{:?}", buffer.to_vec().yin_pitch_detection(SAMPLE_RATE, THRESHOLD));
            { pa::Continue }
    };

    Ok(pa.open_non_blocking_stream(settings, callback)?)
}