use crate::{
    analyze::{Analyze, DetectionResult},
    generation::parsed_to_render::sum_all_waveforms,
    instrument::StereoWaveform,
    renderable::{renderables_to_render_voices, Offset, RenderOp, RenderVoice},
    ring_buffer::RingBuffer,
    settings::{default_settings, Settings},
    write::write_output_buffer,
};
use error::Error;
use portaudio as pa;
use rayon::prelude::*;

fn process_detection_result(result: &mut DetectionResult) -> (f64, f64) {
    if result.gain < 0.005 || result.frequency > 1_000.0 {
        result.frequency = 0.0;
        result.gain = 0.0;
    }

    println!("freq {}, gain {}", result.frequency, result.gain);
    (result.frequency as f64, result.gain as f64)
}

fn sing_along_callback(
    args: pa::DuplexStreamCallbackArgs<'_, f32, f32>,
    input_buffer: &mut RingBuffer<f32>,
    voices: &mut Vec<RenderVoice>,
    settings: &Settings,
) {
    input_buffer.push_vec(args.in_buffer.to_vec());

    let mut detection_result: DetectionResult = input_buffer
        .to_vec()
        .analyze(settings.sample_rate as f32, settings.probability_threshold);

    let (freq, gain) = process_detection_result(&mut detection_result);

    let offset = Offset { freq, gain };

    let result: Vec<StereoWaveform> = voices
        .par_iter_mut()
        .map(|voice| voice.render_batch(1024, Some(&offset)))
        .collect();
    let stereo_waveform = sum_all_waveforms(result);
    write_output_buffer(args.out_buffer, stereo_waveform);
}

pub fn duplex_setup(
    renderables: Vec<Vec<RenderOp>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let settings = default_settings();
    let duplex_stream_settings = get_duplex_settings(&pa, &settings)?;
    let mut voices = renderables_to_render_voices(renderables);

    let mut input_buffer: RingBuffer<f32> = RingBuffer::<f32>::new(settings.yin_buffer_size);

    let duplex_stream = pa.open_non_blocking_stream(duplex_stream_settings, move |args| {
        sing_along_callback(args, &mut input_buffer, &mut voices, &settings);
        pa::Continue
    })?;

    Ok(duplex_stream)
}

fn get_duplex_settings(
    ref pa: &pa::PortAudio,
    ref settings: &Settings,
) -> Result<pa::stream::DuplexSettings<f32, f32>, Error> {
    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    //    println!("Default input device info: {:#?}", &input_info);

    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(
        def_input,
        settings.channels,
        settings.interleaved,
        latency,
    );

    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    //    println!("Default output device info: {:#?}", &output_info);

    let latency = output_info.default_low_output_latency;
    let output_params =
        pa::StreamParameters::new(def_output, settings.channels, settings.interleaved, latency);

    let duplex_settings = pa::DuplexStreamSettings::new(
        input_params,
        output_params,
        settings.sample_rate as f64,
        settings.buffer_size as u32,
    );

    Ok(duplex_settings)
}
