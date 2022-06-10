use crate::{generation::parsed_to_render::sum_all_waveforms, write::write_output_buffer};
use portaudio as pa;
use rayon::prelude::*;
use weresocool_error::Error;
use weresocool_instrument::{renderable::Renderable, RenderOp, RenderVoice};
use weresocool_shared::{default_settings, Settings};

const SETTINGS: Settings = default_settings();

pub fn real_time(
    mut voices: Vec<RenderVoice>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        #[cfg(feature = "app")]
        let iter = voices.par_iter_mut();
        #[cfg(feature = "wasm")]
        let iter = voices.iter_mut();

        let result: Vec<(_, _)> = iter
            // .filter_map(|voice| voice.render_batch(SETTINGS.buffer_size, None))
            .filter_map(|voice| {
                let ops = voice.get_batch(SETTINGS.buffer_size, None);
                match ops {
                    Some(mut batch) => {
                        Some((batch.clone(), batch.render(&mut voice.oscillator, None)))
                    }
                    None => None,
                }
            })
            .collect();
        let (_ops, result): (Vec<_>, Vec<_>) = result.into_iter().map(|(a, b)| (a, b)).unzip();

        if !result.is_empty() {
            let stereo_waveform = sum_all_waveforms(result);
            write_output_buffer(args.buffer, stereo_waveform);
            pa::Continue
        } else {
            pa::Complete
        }
    })?;

    Ok(output_stream)
}

pub fn get_output_settings(pa: &pa::PortAudio) -> Result<pa::stream::OutputSettings<f32>, Error> {
    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    // println!("Default output device info: {:#?}", &output_info);
    let latency = output_info.default_low_output_latency;
    let output_params =
        pa::StreamParameters::new(def_output, SETTINGS.channels, SETTINGS.interleaved, latency);

    let output_settings = pa::OutputStreamSettings::new(
        output_params,
        SETTINGS.sample_rate as f64,
        SETTINGS.buffer_size as u32,
    );

    Ok(output_settings)
}
