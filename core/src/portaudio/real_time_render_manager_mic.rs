use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_instrument::{renderable::Offset, RenderOp, StereoWaveform};

use std::sync::{Arc, Mutex};
use weresocool_analyze::{Analyze, DetectionResult};
use weresocool_error::Error;
use weresocool_portaudio as pa;
use weresocool_ring_buffer::RingBuffer;
use weresocool_shared::Settings;

pub fn real_time_render_manager_mic(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let duplex_stream_settings = get_duplex_settings(&pa)?;
    let buffer_size = Settings::global().buffer_size;

    // TODO!!!!
    let f_basis = 311.127;

    let mut input_buffer: RingBuffer<f32> =
        RingBuffer::<f32>::new(Settings::global().yin_buffer_size);

    let duplex_stream = pa.open_non_blocking_stream(duplex_stream_settings, move |args| {
        input_buffer.push_vec(args.in_buffer.to_vec());

        let mut detection_result: DetectionResult = input_buffer.to_vec().analyze(
            Settings::global().sample_rate as f32,
            Settings::global().probability_threshold,
        );

        let (freq, gain) = process_detection_result(&mut detection_result);

        let batch: Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)> =
            render_manager.lock().unwrap().read(
                buffer_size,
                Offset {
                    freq: freq / f_basis,
                    // gain: gain * 0.1,
                    gain,
                },
            );

        if let Some((b, ramp, _ops)) = batch {
            new_write_output_buffer(args.out_buffer, b, ramp);
            // render_manager.lock().unwrap().push_ops_to_store(ops);
            pa::Continue
        } else {
            write_output_buffer(
                args.out_buffer,
                StereoWaveform::new(Settings::global().buffer_size),
            );

            pa::Continue
        }
    })?;

    Ok(duplex_stream)
}

fn process_detection_result(result: &mut DetectionResult) -> (f64, f64) {
    if result.gain < 0.001 || result.frequency > 2000.0 || result.frequency < 60.0 {
        result.frequency = 0.0;
        result.gain = 0.0;
    }

    // println!("freq {}, gain {}", result.frequency, result.gain);
    (result.frequency as f64, (result.gain * 10.0) as f64)
}

fn get_duplex_settings(pa: &pa::PortAudio) -> Result<pa::stream::DuplexSettings<f32, f32>, Error> {
    let def_input = pa.default_input_device()?;
    // let input_info = pa.device_info(def_input)?;
    // println!("Default input device info: {:#?}", &input_info);

    // let input_latency = input_info.default_low_input_latency;
    // println!("Latency: {}", input_latency);
    // println!("Sample Rate: {}", Settings::global().sample_rate);
    // println!("Buffer Size: {}", Settings::global().buffer_size);
    // println!(
    // "s/b: {}",
    // Settings::global().buffer_size as f32 / Settings::global().sample_rate as f32
    // );
    let input_params = pa::StreamParameters::<f32>::new(
        def_input,
        1,
        Settings::global().interleaved,
        Settings::global().buffer_size as f64 / Settings::global().sample_rate,
    );

    let def_output = pa.default_output_device()?;
    // let output_info = pa.device_info(def_output)?;
    // println!("Default output device info: {:#?}", &output_info);

    // let output_latency = output_info.default_low_output_latency;
    let output_params = pa::StreamParameters::new(
        def_output,
        Settings::global().channels,
        Settings::global().interleaved,
        Settings::global().buffer_size as f64 / Settings::global().sample_rate,
    );

    let duplex_settings = pa::DuplexStreamSettings::new(
        input_params,
        output_params,
        Settings::global().sample_rate,
        Settings::global().buffer_size as u32,
    );

    Ok(duplex_settings)
}
