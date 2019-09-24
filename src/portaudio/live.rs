//use crate::analyze::{DetectionResult};
//use crate::generation::parsed_to_render::*;
use crate::generation::{TimedOp, sum_all_waveforms, generate_waveforms};
use crate::instrument::{Basis, Oscillator, StereoWaveform};
//use crate::ring_buffer::RingBuffer;
use crate::settings::{default_settings, Settings};
//use crate::write::write_output_buffer;
use error::Error;
use num_rational::Rational64;
use portaudio as pa;
use socool_ast::{PointOp, OpOrNfTable};
//use std::iter::Cycle;
//use std::vec::IntoIter;
use crate::portaudio::output::{get_output_settings};

fn live_callback(
    args: pa::OutputStreamCallbackArgs<f32>,
    //state: LiveState,
    basis: Basis,
    settings: &Settings,
) {}

pub struct LiveState {
    pub ops: Vec<TimedOp>,
    pub basis: Basis,
    pub index: usize,
    pub n_voices: usize,
    pub time: Rational64,
}

pub struct LiveRender {
    timed_ops: Vec<Vec<TimedOp>>,
    stereo_waveform: StereoWaveform
}

impl LiveState {
    pub fn new(vec_timed_op: Vec<TimedOp>, n_voices: usize, basis: Basis, settings: &Settings) -> LiveState {
        let mut oscillators: Vec<Oscillator> = vec![];
        for i in 0..n_voices {
            oscillators.push(Oscillator::init(&settings))   
        };

        LiveState {
            ops: vec_timed_op,
            basis,
            n_voices,
            time: Rational64::new(0, 1),
            index: 0,
        }
    }
    pub fn render_batch(&mut self) -> LiveRender {
        let timed_ops = self.get_batch();

        let point_ops: Vec<Vec<PointOp>> = timed_ops.iter().map(|vec| {
            vec.iter().map(|op| {
                op.to_point_op()
            }).collect()
        }).collect();

        let vec_wav = generate_waveforms(&self.basis, point_ops, false);
        let stereo_waveform = sum_all_waveforms(vec_wav);

        LiveRender {
            timed_ops, 
            stereo_waveform
        } 
    }

    pub fn get_batch(&mut self) -> Vec<Vec<TimedOp>> {
        let mut result: Vec<Vec<TimedOp>> = vec![];
        for n in 0..self.n_voices {
            result.push(vec![]);
        }
        let mut remainders: Vec<TimedOp> = vec![];
        let mut search = true;

        self.time += Rational64::new(1,1);
        while search {
            if self.index == self.ops.len() {
                self.index = 0;
                self.time = Rational64::new(0, 1);
            };
            let op = &self.ops[self.index];
            if op.t < self.time {
                let op_end = op.t + op.l;

                if op_end > self.time {
                    let mut shortened = op.clone();
                    let mut remainder = op.clone();

                    shortened.l = self.time - op.t; 
                    remainder.l = op_end - self.time; 
                    remainder.t = self.time; 
                    shortened.next_event = Some(remainder.to_point_op());

                    result[shortened.voice].push(shortened);
                    remainders.push(remainder);
                    self.index += 1;
                } else {
                    result[op.voice].push(op.clone());
                    self.index += 1;
                }
            } else {
                search = false; 
            }
        };
        self.ops = [&remainders[..], &self.ops[..]].concat();
        result
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

