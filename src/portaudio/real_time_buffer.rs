use crate::{
    instrument::StereoWaveform,
    settings::{default_settings, Settings},
    write::write_output_buffer,
};
use portaudio as pa;
use std::sync::{Arc, Mutex};
use weresocool_error::Error;

const SETTINGS: Settings = default_settings();

pub struct RealTimeRender {
    pub stereo_waveform: StereoWaveform,
    write_idx: usize,
    read_idx: usize,
}

/// This assumes that all buffers are the same size
impl RealTimeRender {
    pub fn init() -> Self {
        Self {
            stereo_waveform: StereoWaveform::new(0),
            write_idx: 0,
            read_idx: 0,
        }
    }
    pub fn write(&mut self, stereo_waveform: StereoWaveform) {
        self.stereo_waveform.append(stereo_waveform);
        self.write_idx += 1;
    }
    pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
        let sw = self.stereo_waveform.get_buffer(self.read_idx, buffer_size);
        self.read_idx += 1;
        sw
    }
}

pub fn real_time_buffer(
    real_time_render: Arc<Mutex<RealTimeRender>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        let sw = real_time_render.lock().unwrap().read(SETTINGS.buffer_size);

        match sw {
            Some(stereo_waveform) => {
                write_output_buffer(args.buffer, stereo_waveform);
                pa::Continue
            }
            None => pa::Complete,
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
