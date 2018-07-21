extern crate rand;
use self::rand::Rng;
use event::{generate_test_phrase, Event, Mutate, Phrase};
use new_oscillator::NewOscillator;
//use oscillator::{Oscillator, StereoWaveform};
use portaudio as pa;
use ratios::{mono_ratios, R};
use settings::{get_default_app_settings, Settings};
use std;
use std::sync::mpsc::channel;
use std::sync::Arc;

pub fn setup_portaudio_output(
    ref pa: &pa::PortAudio,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, pa::Error> {
    let settings = get_default_app_settings();

    //    let (l_ratios, r_ratios) = ;
    let mut oscillator = NewOscillator::init(get_default_app_settings());
    let mut freq = 100.0;
    let output_settings = get_output_settings(&pa, &get_default_app_settings())?;

    let mut counter = 0;
    let mut index = 0;
    let test_phrase = generate_test_phrase();

    let output_stream = pa.open_non_blocking_stream(
        output_settings,
        move |pa::OutputStreamCallbackArgs { mut buffer, .. }| {
            let (l_waveform, r_waveform) = oscillator.generate();
            oscillator.update(freq, 1.0);

            //            index = index % (test_phrase.len());
            //
            //            if counter % 25 == 0 {
            //                freq = test_phrase[index].frequency / 1.4;
            //                oscillator.stereo_ratios = test_phrase[index].ratios.clone();
            //                oscillator.gain.past = 0.0;
            //                index += 1;
            //            }

            if counter % 25 == 0 {
                let vs: Vec<f32> = vec![100.0, 500.0, 0.0, 300.0, 0.0, 600.0, 250.0];
                let change = rand::thread_rng().choose(&vs);
                match change {
                    Some(change) => {
                        freq = *change;
                    }
                    _ => {}
                }
            }
            counter += 1;
            write_output_buffer(&mut buffer, l_waveform, r_waveform);
            pa::Continue
        },
    )?;

    Ok(output_stream)
}

fn write_output_buffer(out_buffer: &mut [f32], l_waveform: Vec<f32>, r_waveform: Vec<f32>) {
    let mut l_idx = 0;
    let mut r_idx = 0;
    for n in 0..out_buffer.len() {
        if n % 2 == 0 {
            out_buffer[n] = l_waveform[l_idx];
            l_idx += 1
        } else {
            out_buffer[n] = r_waveform[r_idx];
            r_idx += 1
        }
    }
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
