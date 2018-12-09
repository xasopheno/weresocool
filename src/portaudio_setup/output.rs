use instrument::stereo_waveform::StereoWaveform;
use portaudio as pa;
use settings::{get_default_app_settings, Settings};
use write::write_output_buffer;

pub fn setup_portaudio_output(
    mut composition: StereoWaveform,
    ref pa: &pa::PortAudio,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, pa::Error> {
    let settings = get_default_app_settings();
    let output_settings = get_output_settings(&pa, &settings)?;
    let mut index = 0;
    let output_stream = pa.open_non_blocking_stream(
        output_settings,
        move |pa::OutputStreamCallbackArgs { mut buffer, .. }| {
            let buffer_to_write = composition.get_buffer(index, settings.buffer_size);
            write_output_buffer(&mut buffer, buffer_to_write);
            index += 1;
            pa::Continue
        },
    )?;
    Ok(output_stream)
}

pub fn get_output_settings(
    ref pa: &pa::PortAudio,
    ref settings: &Settings,
) -> Result<pa::stream::OutputSettings<f32>, pa::Error> {
    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    //        println!("Default output device info: {:#?}", &output_info);

    let latency = output_info.default_low_output_latency;
    let output_params =
        pa::StreamParameters::new(def_output, settings.channels, settings.interleaved, latency);

    let output_settings = pa::OutputStreamSettings::new(
        output_params,
        settings.sample_rate as f64,
        settings.buffer_size as u32,
    );

    Ok(output_settings)
}
