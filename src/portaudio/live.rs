use crate::analyze::{DetectionResult};
use crate::generation::parsed_to_render::*;
use crate::instrument::{Basis, Oscillator, StereoWaveform};
use crate::ring_buffer::RingBuffer;
use crate::settings::{default_settings, Settings};
use crate::write::write_output_buffer;
use error::Error;
use num_rational::Rational64;
use portaudio as pa;
use socool_ast::PointOp;
use std::iter::Cycle;
use std::vec::IntoIter;
use crate::portaudio::output::{get_output_settings};

fn live_callback(
    args: pa::OutputStreamCallbackArgs<f32>,
    //state: State,
    basis: Basis,
    settings: &Settings,
) {}

pub struct State {
    pub voices: Vec<Voice>,
    pub basis: Basis,
}

pub struct Voice {
    pub events: Vec<PointOp>,
    pub index: usize,
    pub oscillator: Oscillator,
    pub remainder: Option<PointOp>
}

impl Voice {
    pub fn inc(&mut self) {
        if self.index < self.events.len() - 1 {
            self.index +=  1;
        } else {
            self.index = 0;
        }
    }
    fn generate_waveform(
        &mut self,
        origin: Basis,
    ) -> StereoWaveform {
        //if voice.state.count >= voice.state.current_op.l {
            //voice.state.count = Rational64::new(0, 1);
            //voice.state.current_op = voice.iterator.next().unwrap()
        //}

        //let mut current_point_op = voice.state.current_op.clone();

        //current_point_op.l = Rational64::new(settings.buffer_size as i64, settings.sample_rate as i64);

        //let stereo_waveform = render_mic(&current_point_op, origin, &mut voice.oscillator);
        //voice.state.inc();
        //stereo_waveform
        StereoWaveform::new(2048)
    }
}

pub fn live_setup(
    parsed_composition: Vec<Vec<PointOp>>,
    basis: Basis,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let settings = default_settings();
    let output_settings = get_output_settings(&pa, &settings)?;

    //let mut nf_voice_cycles = setup_iterators(parsed_composition, &settings);

    let live_stream = pa.open_non_blocking_stream(output_settings, move |args| {
        live_callback(args, basis.clone(), &settings);
        pa::Continue
    })?;

    Ok(live_stream)
}

