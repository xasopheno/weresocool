extern crate rand;
use self::rand::Rng;
use oscillator::Oscillator;
use portaudio as pa;
use std;
use std::sync::mpsc::channel;
use std::sync::Arc;
use ratios::{R, simple_ratios, complicated_ratios};
use settings::{get_default_app_settings, Settings};


pub fn setup_portaudio_output(
    ref pa: &pa::PortAudio,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, pa::Error> {
    let settings = get_default_app_settings();

    let (l_ratios, r_ratios) = complicated_ratios();
    let mut oscillator = Oscillator::new(10, l_ratios, r_ratios, get_default_app_settings());
    let mut freq = 100.0;
    oscillator.update(freq, 1.0, 1.0);
    let output_settings = get_output_settings(&pa, &get_default_app_settings())?;

    let mut counter = 0;

    let output_stream = pa.open_non_blocking_stream(output_settings, move |args| {
        let (l_waveform, r_waveform) = oscillator.generate();

        let mut l_idx = 0;
        let mut r_idx = 0;
        for n in 0..args.buffer.len() {
            if n % 2 == 0 {
                args.buffer[n] = l_waveform[l_idx];
                l_idx += 1;
            } else {
                args.buffer[n] = r_waveform[r_idx];
                r_idx += 1
            }
        }

        counter += 1;
//        if counter % 100 == 0 {
//            let vs = vec![6.0, 1.0, -1.0, -2.0, 3.1, 0.0, 0.0];
//            let change = rand::thread_rng().choose(&vs);
//            match change {
//                Some(change) => {
//                    if freq > 110.0 || freq < 40.0 {
//                        freq = 50.0
//                    }
//                    freq += change;
//                }
//                _ => {}
//            }
//        }

        oscillator.update(freq, 1.0, 1.0);
        pa::Continue
    })?;

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
