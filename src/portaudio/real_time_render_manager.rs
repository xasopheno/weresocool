use crate::manager::RenderManager;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
// use web_sys::console;
use weresocool_instrument::StereoWaveform;

use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use weresocool_shared::{default_settings, Settings};

const SETTINGS: Settings = default_settings();

pub fn real_time_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<cpal::Stream, Error> {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    // .expect("failed to find a default output device");

    let array = js_sys::Array::new();
    array.push(&"Hello Console Log".into());
    web_sys::console::log(&array);

    let config = device.default_output_config().unwrap();

    let config = StreamConfig {
        channels: 2,
        buffer_size: BufferSize::Fixed(SETTINGS.buffer_size as u32),
        // buffer_size: BufferSize::Default,
        sample_rate: cpal::SampleRate(SETTINGS.sample_rate as u32),
    };

    // let channels = config.channels as usize;
    // let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());
    let err_fn = |_| {};

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| write_data(data, &render_manager),
            err_fn,
        )
        .unwrap();

    Ok(stream)
}

fn write_data(output: &mut [f32], render_manager: &Arc<Mutex<RenderManager>>) {
    // let array = js_sys::Array::new();
    // array.push(&"Hello Console Log".into());
    // web_sys::console::log(&array);
    let batch: Option<(StereoWaveform, Vec<f32>)> =
        render_manager.lock().unwrap().read(SETTINGS.buffer_size);

    if let Some((b, ramp)) = batch {
        let mut idx = 0;
        for frame in output.chunks_mut(2) {
            frame[0] = ramp[idx] * b.l_buffer[idx] as f32;
            frame[1] = ramp[idx] * b.r_buffer[idx] as f32;
            idx += 1;
        }
    }
}
