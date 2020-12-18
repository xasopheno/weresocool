extern crate anyhow;
extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    // let config = device.default_output_config()?;
    // dbg!(config.buffer_size());
    //
    let config = StreamConfig {
        channels: 2,
        buffer_size: BufferSize::Fixed(1024 * 4),
        sample_rate: cpal::SampleRate(44_100),
    };

    run(&device, &config.into())?;

    Ok(())
}

fn run(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin() * 0.008
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data(output: &mut [f32], channels: usize, next_sample: &mut dyn FnMut() -> f32) {
    dbg!(&output.len());
    for frame in output.chunks_mut(channels) {
        let value: f32 = cpal::Sample::from(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
