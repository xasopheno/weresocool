extern crate rand;
use self::rand::Rng;
use event::{generate_test_phrase, Event, Mutate, Phrase};
use new_oscillator::{NewOscillator, StereoWaveform};
use portaudio as pa;
use settings::{get_default_app_settings, Settings};
use write_output_buffer::{write_output_buffer};
use ratios::{R, Pan, simple_ratios};

pub fn setup_portaudio_output(
    ref pa: &pa::PortAudio,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, pa::Error> {
    let settings = get_default_app_settings();
    let r = vec![
        R::atio(1, 1, 0.0, 0.8, Pan::Right),
        R::atio(1, 1, 3.0, 0.8, Pan::Left),
    ];
    let mut oscillator = NewOscillator::init(r, &settings);
    let mut freq = 0.0;
    let output_settings = get_output_settings(&pa, &settings)?;
    let mut index = 0;
    let mut stereo_waveform = generate_test_phrase();
    let output_stream = pa.open_non_blocking_stream(
        output_settings,
        move |pa::OutputStreamCallbackArgs { mut buffer, .. }| {

            let buffer_to_write = stereo_waveform.get_buffer(index, settings.buffer_size);
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
    //    println!("Default output device info: {:#?}", &output_info);

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
