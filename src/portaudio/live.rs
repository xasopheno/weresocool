//use crate::analyze::{DetectionResult};
//use crate::generation::parsed_to_render::*;
use crate::generation::{generate_waveforms, sum_all_waveforms, TimedOp};
use crate::instrument::{Basis, Oscillator, StereoWaveform};
//use crate::ring_buffer::RingBuffer;
use crate::settings::{default_settings, Settings};
use crate::write::write_output_buffer;
//use crate::write::write_output_buffer;
use error::Error;
use num_rational::Rational64;
use portaudio as pa;
use socool_ast::{OpOrNfTable, PointOp};
use crate::render::Render;
use rayon::prelude::*;
//use std::iter::Cycle;
//use std::vec::IntoIter;
use crate::portaudio::output::get_output_settings;

pub struct LiveState {
    pub settings: Settings,
    pub ops: Vec<TimedOp>,
    pub oscillators: Vec<Oscillator>,
    pub basis: Basis,
    pub index: usize,
    pub n_voices: usize,
    pub time: Rational64,
}

pub struct LiveRender {
    timed_ops: Vec<Vec<TimedOp>>,
    stereo_waveform: StereoWaveform,
    index: usize,
}

impl LiveRender {
    fn render_all(&mut self) {}
}

impl LiveState {
    pub fn new(
        vec_timed_op: Vec<TimedOp>,
        n_voices: usize,
        basis: Basis,
        settings: &Settings,
    ) -> LiveState {
        let mut oscillators: Vec<Oscillator> = vec![];
        for i in 0..n_voices {
            oscillators.push(Oscillator::init(&settings))
        }

        LiveState {
            settings: default_settings(),
            oscillators,
            ops: vec_timed_op,
            basis,
            n_voices,
            time: Rational64::new(0, 1),
            index: 0,
        }
    }
    pub fn render_batch(&mut self) -> LiveRender {
        let timed_ops = self.get_batch();

        let mut point_ops: Vec<Vec<PointOp>> = timed_ops
            .iter()
            .map(|vec| vec.iter().map(|op| op.to_point_op()).collect())
            .collect();

        let vec_v: Vec<StereoWaveform> = point_ops
            .iter_mut()
            .enumerate()
            .map(|(n, ref mut vec_point_op)| {
                dbg!(&vec_point_op);
                let mut osc = &mut self.oscillators[n];
                vec_point_op.render(&self.basis, &mut osc)
            })
            .collect();

        //let vec_wav = generate_waveforms(&self.basis, point_ops, false);
        let stereo_waveform = sum_all_waveforms(vec_v);

        LiveRender {
            timed_ops,
            stereo_waveform,
            index: 0,
        }
    }

    fn render_vec_timed_op(t_ops: Vec<TimedOp>, origin: &Basis, oscillator: &mut Oscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut t_ops = t_ops.clone();
        //t_ops.push(PointOp::init_silent());

        let mut iter = t_ops.iter().peekable();

        while let Some(t_op) = iter.next() {
            let mut next_op = None;
            let peek = iter.peek();
            match peek {
                Some(p) => next_op = Some(p.clone().clone()),
                None => {}
            };

            //TODO: HERE
            //let stereo_waveform = t_op.clone().render(origin, oscillator, next_op);
            //result.append(stereo_waveform);
        }

        result
    }

    //fn render_timed_op(
        //&mut self,
        //origin: &Basis,
        //oscillator: &mut Oscillator,
        //next_op: Option<PointOp>,
    //) -> StereoWaveform {
        //oscillator.update(origin.clone(), self, next_op);

        //let n_samples_to_generate = r_to_f64(self.l) * origin.l * 44_100.0;
        //let portamento_length = r_to_f64(self.portamento);

        //oscillator.generate(n_samples_to_generate, portamento_length)
    //}


    pub fn get_batch(&mut self) -> Vec<Vec<TimedOp>> {
        let mut result: Vec<Vec<TimedOp>> = vec![];
        for n in 0..self.n_voices {
            result.push(vec![]);
        }
        let mut remainders: Vec<TimedOp> = vec![];
        let mut search = true;

        self.time += Rational64::new(
            self.settings.buffer_size as i64,
            self.settings.sample_rate as i64,
        );

        while search {
            if self.index == self.ops.len() { 
                break
                //self.index = 0;
                //self.time = Rational64::new(
                    //self.settings.buffer_size as i64,
                    //self.settings.sample_rate as i64,
                //);
            };
            let op = &self.ops[self.index];
            if op.t < self.time {
                let op_end = op.t + op.l;

                if op_end >= self.time {
                    let mut shortened = op.clone();
                    let mut remainder = op.clone();

                    shortened.l = self.time - op.t;
                    remainder.l = op_end - self.time;
                    remainder.t = self.time;
                    dbg!(shortened.l);
                    dbg!(shortened.t);
                    dbg!(remainder.l);
                    dbg!(remainder.t);

                    shortened.next_event = Some(remainder.to_point_op());

                    result[shortened.voice].push(shortened);
                    remainders.push(remainder);
                    self.index += 1;
                } else {
                    result[op.voice].push(op.clone());
                    self.index += 1;
                }
            } else {
                self.index -= 1;
                search = false;
            }
        }
        //self.ops = [&remainders[..], &self.ops[..]].concat();
        let x: Vec<TimedOp> = self.ops.splice(self.index..self.index, remainders).collect();
        //for (i, op) in remainders.iter().enumerate() {
            //self.ops.insert(self.index + i - 1, op.clone());
        //}
        result
    }
}

pub fn live_setup(
    mut state: LiveState,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let settings = default_settings();
    let output_settings = get_output_settings(&pa, &settings)?;

    let mut result: Vec<LiveRender> = vec![];
    for i in 0..1 {
        let live_render = state.render_batch();
        result.push(live_render);
    }
    let mut buffer_to_write = result.into_iter();
    let live_stream =
        pa.open_non_blocking_stream(output_settings, move |args| match buffer_to_write.next() {
            Some(result) => {
                write_output_buffer(args.buffer, result.stereo_waveform.clone());
                pa::Continue
            }
            None => pa::Complete,
        })?;

    Ok(live_stream)
}
