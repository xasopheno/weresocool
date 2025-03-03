use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};
use weresocool_instrument::{renderable::Offset, StereoWaveform};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex};
use weresocool_error::Error;
use cpal::{traits::{DeviceTrait, HostTrait}, StreamConfig};
use crate::manager::VisualizationChannel;
use weresocool_shared::Settings;
use log::info;


pub fn real_time_render_manager_new(
    vis_event_sender: VisualizationChannel,
    mic_receiver_sync: Receiver<(f32, f32)>,
    basis_f: f32,
) -> Result<(Arc<Mutex<RenderManager>>, cpal::Stream), Error> {
    let render_manager = Arc::new(Mutex::new(RenderManager::init(vis_event_sender, None, false, None)));

    let settings = Settings::global();
    let sample_rate = cpal::SampleRate(settings.sample_rate as u32);
    let buffer_size = cpal::BufferSize::Fixed(settings.buffer_size as u32);


    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("Failed to get default output device");

    let output_config = StreamConfig {
        channels: settings.channels as u16,
        sample_rate,
        buffer_size,
    };

    dbg!(&output_config);
    info!("Output sample_rate: {:?}", output_config.sample_rate.0);
    info!("Output buffer_size: {:?}", output_config.buffer_size);

    let render_manager_clone = Arc::clone(&render_manager);
    let mic_receiver_sync = Arc::new(Mutex::new(mic_receiver_sync));

    let output_stream = output_device.build_output_stream(
        &output_config,
        move |output: &mut [f32], _| {
            let mut render_manager = render_manager_clone.lock().unwrap();
            let mic_receiver = mic_receiver_sync.lock().unwrap();

            if let Ok((freq, gain)) = mic_receiver.try_recv() {

                if let Some((b, ramp, _ops)) = render_manager.read(
                    output.len() / 2,
                    Offset {
                        freq: (freq / basis_f) as f64,
                        gain: gain as f64,
                    },
                ) {
                    new_write_output_buffer(output, b, ramp);
                } else {
                    write_output_buffer(output, StereoWaveform::new(output.len() / 2));
                }
            } else {
                write_output_buffer(output, StereoWaveform::new(output.len() / 2));
            }
        },
        |err| eprintln!("Output stream error: {}", err),
        None,
    ).unwrap();

    Ok((render_manager, output_stream))
}

