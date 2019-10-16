use crate::analyze::{Analyze, DetectionResult};
use crate::generation::{
    parsed_to_render::{render_mic, sum_all_waveforms},
    renderable::RenderOp,
};
use crate::instrument::{Oscillator, StereoWaveform};
use crate::ring_buffer::RingBuffer;
use crate::settings::{default_settings, Settings};
use crate::write::write_output_buffer;
use error::Error;
use portaudio as pa;
use std::iter::Cycle;
use std::vec::IntoIter;

pub struct RealTimeState {
    count: f64,
    inc: f64,
    current_op: RenderOp,
}

impl RealTimeState {
    fn inc(&mut self) {
        self.count += self.inc
    }
}

fn process_detection_result(result: &mut DetectionResult) -> (f64, f64) {
    if result.gain < 0.005 || result.frequency > 1_000.0 {
        result.frequency = 0.0;
        result.gain = 0.0;
    }

    println!("freq {}, gain {}", result.frequency, result.gain);
    (result.frequency as f64, result.gain as f64)
}

pub struct NfVoiceState {
    oscillator: Oscillator,
    state: RealTimeState,
    iterator: Cycle<IntoIter<RenderOp>>,
}

pub fn setup_iterators(
    parsed_composition: Vec<Vec<RenderOp>>,
    settings: &Settings,
) -> Vec<NfVoiceState> {
    let mut nf_voice_cycles = vec![];

    for seq in parsed_composition {
        let mut iterator = seq.clone().into_iter().cycle();
        let state = RealTimeState {
            count: 0.0,
            inc: settings.buffer_size as f64 / settings.sample_rate as f64,
            current_op: iterator
                .next()
                .expect("Empty iterator in cycle in mic. Empty?"),
        };

        nf_voice_cycles.push(NfVoiceState {
            oscillator: Oscillator::init(&default_settings()),
            iterator,
            state,
        });
    }

    nf_voice_cycles
}

fn sing_along_callback(
    args: pa::DuplexStreamCallbackArgs<'_, f32, f32>,
    input_buffer: &mut RingBuffer<f32>,
    nf_voice_cycles: &mut Vec<NfVoiceState>,
    basis_f: f64,
    settings: &Settings,
) {
    input_buffer.push_vec(args.in_buffer.to_vec());

    let mut detection_result: DetectionResult = input_buffer
        .to_vec()
        .analyze(settings.sample_rate as f32, settings.probability_threshold);

    let (freq, gain) = process_detection_result(&mut detection_result);
    let freq_ratio = freq / basis_f;

    let result: Vec<StereoWaveform> = nf_voice_cycles
        .iter_mut()
        .map(|voice| generate_voice_sw(voice, settings, freq_ratio, gain))
        .collect();

    let stereo_waveform = sum_all_waveforms(result);
    write_output_buffer(args.out_buffer, stereo_waveform);
}

fn generate_voice_sw(
    voice: &mut NfVoiceState,
    settings: &Settings,
    freq_ratio: f64,
    gain: f64,
) -> StereoWaveform {
    if voice.state.count >= voice.state.current_op.l {
        voice.state.count = 0.0;
        voice.state.current_op = voice.iterator.next().unwrap()
    }

    let mut current_op = voice.state.current_op.clone();
    current_op.f *= freq_ratio;
    current_op.g = (current_op.g.0 * gain / 2.0, current_op.g.1 * gain / 2.0);

    current_op.l = settings.buffer_size as f64 / settings.sample_rate as f64;

    let stereo_waveform = render_mic(&current_op, &mut voice.oscillator);
    voice.state.inc();
    stereo_waveform
}

pub fn duplex_setup(
    parsed_composition: Vec<Vec<RenderOp>>,
    basis_f: f64,
) -> Result<pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let settings = default_settings();
    let duplex_stream_settings = get_duplex_settings(&pa, &settings)?;

    let mut input_buffer: RingBuffer<f32> = RingBuffer::<f32>::new(settings.yin_buffer_size);
    let mut nf_voice_cycles = setup_iterators(parsed_composition, &settings);

    let mut count = 0;

    let duplex_stream = pa.open_non_blocking_stream(duplex_stream_settings, move |args| {
        if count < 20 {
            count += 1;
            if count == 20 {
                println!("* * * * * ready * * * * *");
            }
            pa::Continue
        } else {
            sing_along_callback(
                args,
                &mut input_buffer,
                &mut nf_voice_cycles,
                basis_f,
                &settings,
            );
            pa::Continue
        }
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
