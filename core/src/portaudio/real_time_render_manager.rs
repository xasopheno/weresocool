use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_instrument::StereoWaveform;

use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use weresocool_portaudio as pa;
use weresocool_shared::Settings;
use weresocool_visualizer::{WereSoCoolSpectrum, WereSoCoolSpectrumConfig};

pub fn real_time_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        let batch: Option<(StereoWaveform, Vec<f32>)> = render_manager
            .lock()
            .unwrap()
            .read(Settings::global().buffer_size);

        if let Some((b, ramp)) = batch {
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

pub fn real_time_audio_visual_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<
    (
        pa::Stream<pa::NonBlocking, pa::Output<f32>>,
        WereSoCoolSpectrum,
    ),
    Error,
> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;
    let buffer_size = Settings::global().buffer_size;
    let config = WereSoCoolSpectrumConfig::new(buffer_size);
    let (visualizer, (fft_sender_l, fft_sender_r)) = WereSoCoolSpectrum::new(&config).unwrap();

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        let batch: Option<(StereoWaveform, Vec<f32>)> =
            render_manager.lock().unwrap().read(buffer_size);

        if let Some((b, ramp)) = batch {
            _ = fft_sender_l.send(b.l_buffer.clone().into_iter().map(|x| x as f32).collect());
            _ = fft_sender_r.send(b.r_buffer.clone().into_iter().map(|x| x as f32).collect());
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

    Ok((output_stream, visualizer))
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
        Settings::global().sample_rate as f64,
        Settings::global().buffer_size as u32,
    );

    Ok(output_settings)
}
