use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use weresocool_instrument::StereoWaveform;

use portaudio as pa;
use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use weresocool_shared::{default_settings, Settings};

const SETTINGS: Settings = default_settings();

pub fn real_time_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
    // ) -> Result<cpal::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
) -> Result<cpal::Stream, Error> {
    // let pa = pa::PortAudio::new()?;
    // let output_stream_settings = get_output_settings(&pa)?;
    // let mut x = 0;
    //
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");

    // cpal::available_hosts()
    // .into_iter()
    // .map(|id| {
    // dbg!(id);
    // 1
    // })
    // .collect::<Vec<usize>>();
    // panic!();
    // let config = device.default_output_config()?;
    // dbg!(config.buffer_size());
    //
    let config = StreamConfig {
        channels: 2,
        buffer_size: BufferSize::Fixed(SETTINGS.buffer_size as u32),
        // buffer_size: BufferSize::Default,
        sample_rate: cpal::SampleRate(44_100),
    };

    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, render_manager.clone())
            },
            err_fn,
        )
        .unwrap();

    Ok(stream)
}

fn write_data(output: &mut [f32], channels: usize, render_manager: Arc<Mutex<RenderManager>>) {
    let (batch, _ramp) = render_manager.lock().unwrap().read(output.len()).unwrap();
    let mut l_idx = 0;
    let mut r_idx = 0;
    for frame in output.chunks_mut(channels) {
        frame[0] = batch.l_buffer[l_idx] as f32;
        frame[1] = batch.r_buffer[r_idx] as f32;
        l_idx += 1;
        r_idx += 1;
    }
}
