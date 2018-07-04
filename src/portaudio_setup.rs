extern crate rand;
use portaudio as pa;
use std;
use oscillator::Oscillator;
use ring_buffer::{RingBuffer};
use std::sync::Arc;
use analyze::{Analyze};
use settings::{get_default_app_settings, Settings};

pub fn setup_portaudio_duplex(
    ref pa: &pa::PortAudio,
    oscillator: Arc<std::sync::Mutex<Oscillator>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, pa::Error> {
    let settings = get_default_app_settings();
    let duplex_settings = get_duplex_settings(&pa, &settings)?;
    let mut input_buffer: RingBuffer<f32> = RingBuffer::<f32>::new(settings.yin_buffer_size as usize);
    let duplex_stream = pa.open_non_blocking_stream(
        duplex_settings,
        move |pa::DuplexStreamCallbackArgs{ in_buffer, out_buffer, frames, time, .. }| {

            let mut osc = oscillator.lock().unwrap();

            // *********** input ************

            input_buffer.push_vec(in_buffer.to_vec());
            let mut buffer_vec: Vec<f32> = input_buffer.to_vec();
            let gain = buffer_vec.gain();
            let (frequency, probability) =
            buffer_vec.yin_pitch_detection(settings.sample_rate, settings.threshold);
            //                println!("{}, {}", frequency, probability);

            osc.update(frequency, gain, probability);


            // *********** output ************

            let settings_clone = settings.clone();
            let (l_waveform, r_waveform) = osc.generate(
                settings_clone.output_buffer_size as usize,
                settings_clone.sample_rate,
            );

            let mut l_idx = 0;
            let mut r_idx = 0;
            for n in 0..out_buffer.len() {
                if n % 2 == 0 {
                    out_buffer[n] = l_waveform[l_idx];
                    l_idx += 1;
                } else {
                    out_buffer[n] = r_waveform[r_idx];
                    r_idx += 1
                }
            }

        pa::Continue
})?;

Ok(duplex_stream)

}

fn get_duplex_settings(
    ref pa: &pa::PortAudio,
    ref settings: &Settings,
) -> Result<pa::stream::DuplexSettings<f32, f32>, pa::Error> {

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
            settings.output_buffer_size as u32,
        );

        Ok(duplex_settings)
}
