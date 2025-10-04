use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_instrument::{renderable::Offset, StereoWaveform};

use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use weresocool_portaudio as pa;
use weresocool_shared::Settings;

pub fn real_time_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;

    let use_lookahead = Settings::global().lookahead_buffers > 0;

    if use_lookahead {
        // Use background rendering with lookahead buffers
        let _render_thread = RenderManager::start_background_rendering(Arc::clone(&render_manager));

        let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
            // Try to pop a pre-rendered buffer from the queue
            let buffer = render_manager.lock().unwrap().pop_buffer();

            if let Some(prerendered) = buffer {
                new_write_output_buffer(args.buffer, prerendered.waveform, prerendered.ramp);
                pa::Continue
            } else {
                // No buffer available (underrun) - output silence
                write_output_buffer(
                    args.buffer,
                    StereoWaveform::new(Settings::global().buffer_size),
                );

                pa::Continue
            }
        })?;

        Ok(output_stream)
    } else {
        // Original behavior: render directly on audio thread (no lookahead)
        let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
            let batch = render_manager.lock().unwrap().read(
                Settings::global().buffer_size,
                Offset {
                    freq: 1.0,
                    gain: 1.0,
                },
            );

            if let Some((b, ramp, _ops)) = batch {
                new_write_output_buffer(args.buffer, b, ramp);
                pa::Continue
            } else {
                write_output_buffer(
                    args.buffer,
                    StereoWaveform::new(Settings::global().buffer_size),
                );

                pa::Continue
            }
        })?;

        Ok(output_stream)
    }
}

pub fn get_output_settings(pa: &pa::PortAudio) -> Result<pa::stream::OutputSettings<f32>, Error> {
    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    // println!("Default output device info: {:#?}", &output_info);
    let latency = output_info.default_low_output_latency;
    let output_params = pa::StreamParameters::new(
        def_output,
        Settings::global().channels,
        Settings::global().interleaved,
        latency,
    );

    let output_settings = pa::OutputStreamSettings::new(
        output_params,
        Settings::global().sample_rate,
        Settings::global().buffer_size as u32,
    );

    Ok(output_settings)
}
