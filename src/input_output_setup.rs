use portaudio as pa;
use settings::Settings;
use std;
use ring_buffer::RingBuffer;
use portaudio_setup::{ setup_portaudio_input, setup_portaudio_output };

pub struct Input {
    pub stream: pa::Stream<pa::stream::NonBlocking, pa::stream::Input<f32>>,
    pub callback_rx: std::sync::mpsc::Receiver<Vec<f32>>,
    pub buffer: RingBuffer<f32>,
}

pub struct Output {
    pub stream: pa::Stream<pa::stream::NonBlocking, pa::stream::Output<f32>>,
    pub frequency: &'static f32,
}

pub fn prepare_input(ref pa: &pa::PortAudio, ref settings: &Settings) -> Result<Input, pa::Error> {

    let (input_stream, input_callback_rx) = setup_portaudio_input(&pa, settings)?;

    let mut input_buffer: RingBuffer<f32> =
        RingBuffer::<f32>::new(settings.yin_buffer_size as usize);

    let mut input = Input{
        stream: input_stream,
        callback_rx: input_callback_rx,
        buffer: input_buffer
    };

    Ok(input)
}

pub fn prepare_output(ref pa: &pa::PortAudio, ref settings: &Settings) -> Result<Output, pa::Error> {
    let output_stream = setup_portaudio_output(&pa, settings)?;

    let mut output = Output{
        stream: output_stream,
    };

    Ok(output)
}