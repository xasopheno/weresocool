////use crate::analyze::{DetectionResult};
////use crate::generation::parsed_to_render::*;
//use crate::generation::{generate_waveforms, r_to_f64, sum_all_waveforms, TimedOp};
//use crate::instrument::{Basis, Oscillator, StereoWaveform};
////use crate::ring_buffer::RingBuffer;
//use crate::settings::{default_settings, Settings};
//use crate::write::write_output_buffer;
////use crate::write::write_output_buffer;
//use crate::render::Render;
//use error::Error;
//use num_rational::Rational64;
//use portaudio as pa;
//use rayon::prelude::*;
//use socool_ast::{OpOrNfTable, PointOp};
////use std::iter::Cycle;
////use std::vec::IntoIter;
//use crate::portaudio::output::get_output_settings;

//pub struct LiveState {
    //pub settings: Settings,
    //pub ops: Vec<TimedOp>,
    //pub ops_bak: Vec<TimedOp>,
    //pub oscillators: Vec<Oscillator>,
    //pub basis: Basis,
    //pub index: usize,
    //pub n_voices: usize,
    //pub time: Rational64,
//}

//pub struct LiveRender {
    //timed_ops: Vec<Vec<TimedOp>>,
    //stereo_waveform: StereoWaveform,
    //index: usize,
//}

//impl LiveRender {
    //fn render_all(&mut self) {}
//}

//impl LiveState {
    //pub fn new(
        //vec_timed_op: Vec<TimedOp>,
        //n_voices: usize,
        //basis: Basis,
        //settings: &Settings,
    //) -> LiveState {
        //let mut oscillators: Vec<Oscillator> = vec![];
        //for i in 0..n_voices {
            //oscillators.push(Oscillator::init(&settings))
        //}
        //let mut vec_timed_op = vec_timed_op.clone();
        //vec_timed_op.reverse();

        //LiveState {
            //settings: default_settings(),
            //oscillators,
            //ops: vec_timed_op.clone(),
            //ops_bak: vec_timed_op.clone(),
            //basis,
            //n_voices,
            //time: Rational64::new(0, 1),
            //index: 0,
        //}
    //}

    //pub fn render_batch(&mut self) -> Option<LiveRender> {
        //match self.get_batch() {
            //Some(timed_ops) => {
                //let mut point_ops: Vec<Vec<PointOp>> = timed_ops
                    //.iter()
                    //.map(|vec| vec.iter().map(|op| op.to_point_op()).collect())
                    //.collect();

                //let vec_v: Vec<StereoWaveform> = point_ops
                    //.iter_mut()
                    //.enumerate()
                    //.map(|(n, vec_point_op)| {
                        //let mut osc = &mut self.oscillators[n];
                        //render_vec_point_op(vec_point_op.clone(), &self.basis, &mut osc)
                    //})
                    //.collect();

                //let stereo_waveform = sum_all_waveforms(vec_v);

                //Some(LiveRender {
                    //timed_ops,
                    //stereo_waveform,
                    //index: 0,
                //})
            //}
            //None => None,
        //}
    //}

    //pub fn get_batch(&mut self) -> Option<Vec<Vec<TimedOp>>> {
        //let mut result: Vec<Vec<TimedOp>> = vec![];
        //for n in 0..self.n_voices {
            //result.push(vec![]);
        //}
        //let mut remainders: Vec<TimedOp> = vec![];
        //let mut search = true;

        //let mut max_time = self.time;

        //self.time += Rational64::new(
            //self.settings.buffer_size as i64,
            //self.settings.sample_rate as i64,
        //);

        //while search {
            ////if self.ops.len() == 0 {
            ////self.ops = self.ops_bak.clone();
            //////self.time = self.time - max_time;
            ////self.time = Rational64::new(0,1);
            ////search = false;
            ////}
            //let op = &self.ops.pop().unwrap();

            //if op.t < self.time {
                //max_time = op.t;
                //let op_end = op.t + op.l;

                //if op_end >= self.time {
                    //let mut shortened = op.clone();
                    //let mut remainder = op.clone();

                    //shortened.l = self.time - op.t;
                    //remainder.l = op_end - self.time;
                    //remainder.t = self.time;

                    //shortened.next_event = Some(remainder.to_point_op());

                    //result[shortened.voice].push(shortened);
                    //remainders.push(remainder);
                //} else {
                    //result[op.voice].push(op.clone());
                //}
            //} else {
                ////self.index -= 1;
                //remainders.push(op.clone());
                //search = false;
            //}
        //}

        //remainders.reverse();
        //self.ops.extend(remainders);

        //if result.len() == 0 {
            //None
        //} else {
            //Some(result)
        //}
    //}
//}

//fn render_vec_point_op(
    //point_ops: Vec<PointOp>,
    //origin: &Basis,
    //oscillator: &mut Oscillator,
//) -> StereoWaveform {
    //let mut result: StereoWaveform = StereoWaveform::new(0);
    //let mut point_ops = point_ops.clone();
    ////point_ops.push(PointOp::init_silent());

    //let mut iter = point_ops.iter().peekable();

    //let mut total_samples = 0.0;
    //while let Some(t_op) = iter.next() {
        //let mut next_op = None;
        //let peek = iter.peek();
        //match peek {
            //Some(p) => next_op = Some(p.clone().clone()),
            //None => {}
        //};
        //let point_op = t_op;
        //oscillator.update(origin.clone(), &point_op, next_op);

        //let n_samples_to_generate = (r_to_f64(point_op.l) * 44_100.0).round();
        //total_samples += n_samples_to_generate;
        //let portamento_length = r_to_f64(point_op.portamento);

        //let sw = oscillator.generate(n_samples_to_generate, 1024.0);

        ////let stereo_waveform = render_timedop(t_op, origin, oscillator, next_op);
        //result.append(sw);
    //}

    //result
//}

//pub fn live_setup(
    //mut state: LiveState,
//) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    //let pa = pa::PortAudio::new()?;
    //let settings = default_settings();
    //let output_settings = get_output_settings(&pa, &settings)?;

    //let live_stream =
        //pa.open_non_blocking_stream(output_settings, move |args| match state.render_batch() {
            //Some(result) => {
                //if result.stereo_waveform.r_buffer.len() == 0 {
                    //return pa::Continue;
                //};
                //dbg!(result.stereo_waveform.r_buffer.len());
                //write_output_buffer(args.buffer, result.stereo_waveform.clone());
                //pa::Continue
            //}
            //None => pa::Continue,
        //})?;

    //Ok(live_stream)
//}
