use portaudio as pa;
use portaudio_setup::{setup_portaudio_input, setup_portaudio_output};
use ring_buffer::RingBuffer;
use settings::Settings;
use std;
use std::sync::atomic::{AtomicIsize};


pub struct Input {
    pub stream: pa::Stream<pa::stream::NonBlocking, pa::stream::Input<f32>>,
    pub callback_rx: std::sync::mpsc::Receiver<Vec<f32>>,
    pub buffer: RingBuffer<f32>,
}

pub struct Output {
    pub stream: pa::Stream<pa::stream::NonBlocking, pa::stream::Output<f32>>,
    pub oscillator: Oscillator,
}

pub struct Oscillator {
    pub frequency: AtomicIsize,
    pub phase: f32,
    pub generator:
        fn(frequency: AtomicIsize, phase: f32, buffer_size: usize, sample_rate: f32) -> Vec<f32>,
}

pub fn prepare_input(ref pa: &pa::PortAudio, ref settings: &Settings) -> Result<Input, pa::Error> {
    let (input_stream, input_callback_rx) =
        setup_portaudio_input(&pa, settings)?;

    let mut input_buffer: RingBuffer<f32> =
        RingBuffer::<f32>::new(settings.yin_buffer_size as usize);

    let input = Input {
        stream: input_stream,
        callback_rx: input_callback_rx,
        buffer: input_buffer,
    };

    Ok(input)
}
