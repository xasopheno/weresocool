use crate::{generation::parsed_to_render::sum_all_waveforms, write::write_output_buffer};
use portaudio as pa;
use rayon::prelude::*;
use weresocool_analyze::{Analyze, DetectionResult};
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    renderables_to_render_voices, Offset, RenderOp, RenderVoice,
};
use weresocool_instrument::StereoWaveform;
use weresocool_ring_buffer::RingBuffer;
use weresocool_shared::{default_settings, Settings};

const SETTINGS: Settings = default_settings();

fn process_detection_result(result: &mut DetectionResult) -> (f64, f64) {
    if result.gain < 0.005 || result.frequency > 1_000.0 {
        result.frequency = 0.0;
        result.gain = 0.0;
    }

    //println!("freq {}, gain {}", result.frequency, result.gain);
    (result.frequency as f64, result.gain as f64)
}

fn sing_along_callback(
    basis_f: f64,
    args: pa::DuplexStreamCallbackArgs<'_, f32, f32>,
    input_buffer: &mut RingBuffer<f32>,
    voices: &mut Vec<RenderVoice>,
) {
    input_buffer.push_vec(args.in_buffer.to_vec());

    let mut detection_result: DetectionResult = input_buffer
        .to_vec()
        .analyze(SETTINGS.sample_rate as f32, SETTINGS.probability_threshold);

    let (freq, gain) = process_detection_result(&mut detection_result);

    let offset = if SETTINGS.mic {
        Some(Offset {
            freq: freq / basis_f,
            gain,
        })
    } else {
        None
    };

    let result: Vec<StereoWaveform> = voices
        .par_iter_mut()
        .filter_map(|voice| voice.render_batch(SETTINGS.buffer_size, offset.as_ref()))
        .collect();
    let stereo_waveform = sum_all_waveforms(result);
    write_output_buffer(args.out_buffer, stereo_waveform);
}

pub fn duplex_setup(
    basis_f: f64,
    renderables: Vec<Vec<RenderOp>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let settings = default_settings();
    let duplex_stream_settings = get_duplex_settings(&pa)?;
    let mut voices = renderables_to_render_voices(renderables);

    let mut input_buffer: RingBuffer<f32> = RingBuffer::<f32>::new(settings.yin_buffer_size);

    let duplex_stream = pa.open_non_blocking_stream(duplex_stream_settings, move |args| {
        sing_along_callback(basis_f, args, &mut input_buffer, &mut voices);
        pa::Continue
    })?;

    Ok(duplex_stream)
}

fn get_duplex_settings(pa: &pa::PortAudio) -> Result<pa::stream::DuplexSettings<f32, f32>, Error> {
    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    //    println!("Default input device info: {:#?}", &input_info);

    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(
        def_input,
        SETTINGS.channels,
        SETTINGS.interleaved,
        latency,
    );

    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    //    println!("Default output device info: {:#?}", &output_info);

    let latency = output_info.default_low_output_latency;
    let output_params =
        pa::StreamParameters::new(def_output, SETTINGS.channels, SETTINGS.interleaved, latency);

    let duplex_settings = pa::DuplexStreamSettings::new(
        input_params,
        output_params,
        SETTINGS.sample_rate as f64,
        SETTINGS.buffer_size as u32,
    );

    Ok(duplex_settings)
}
