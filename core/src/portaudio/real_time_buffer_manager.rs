use crate::{manager::BufferManager, write::write_output_buffer};
use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use weresocool_instrument::StereoWaveform;
use weresocool_portaudio as pa;
use weresocool_shared::Settings;

pub fn real_time_buffer_manager(
    buffer_manager: Arc<Mutex<BufferManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        let sw = buffer_manager
            .lock()
            .unwrap()
            .read(Settings::global().buffer_size);

        match sw {
            Some(stereo_waveform) => {
                write_output_buffer(args.buffer, stereo_waveform);
                pa::Continue
            }
            None => {
                write_output_buffer(
                    args.buffer,
                    StereoWaveform::new(Settings::global().buffer_size),
                );

                pa::Continue
            }
        }
    })?;

    Ok(output_stream)
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
