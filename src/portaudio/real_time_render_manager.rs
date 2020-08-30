use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_instrument::StereoWaveform;

use portaudio as pa;
use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use weresocool_shared::{default_settings, Settings};

const SETTINGS: Settings = default_settings();

pub fn real_time_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        let batch: Option<(StereoWaveform, Vec<f32>)> =
            render_manager.lock().unwrap().read(SETTINGS.buffer_size);

        if let Some((b, ramp)) = batch {
            new_write_output_buffer(args.buffer, b, ramp);
            pa::Continue
        } else {
            write_output_buffer(args.buffer, StereoWaveform::new(SETTINGS.buffer_size));

            pa::Continue
        }
    })?;

    Ok(output_stream)
}

pub fn get_output_settings(pa: &pa::PortAudio) -> Result<pa::stream::OutputSettings<f32>, Error> {
    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    let latency = output_info.default_low_output_latency;
    // println!("Default output device info: {:#?}", &def_output);
    let output_params =
        pa::StreamParameters::new(def_output, SETTINGS.channels, SETTINGS.interleaved, latency);

    let output_settings = pa::OutputStreamSettings::new(
        output_params,
        SETTINGS.sample_rate as f64,
        SETTINGS.buffer_size as u32,
    );

    Ok(output_settings)
}
