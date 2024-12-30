use crate::generation::Normalizer;
use crate::manager::resizeable_2d_vec::Resizeable2DVec;
use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    generation::sum_all_waveforms,
    generation::Op4D,
    interpretable::{InputType, Interpretable},
};
use log::info;
use opmap::OpMap;
use std::sync::mpsc::Sender;
use std::{path::PathBuf, sync::mpsc::SendError};
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, renderables_to_render_voices, Offset, RenderOp, RenderVoice, Renderable,
};
use weresocool_instrument::StereoWaveform;
use weresocool_shared::Settings;

pub type KillChannel = Option<Sender<bool>>;

#[derive(Debug, Clone)]
pub enum VisEvent {
    Ops(opmap::OpMap<Op4D>),
    Reset,
}
pub type VisualizationChannel = Option<crossbeam_channel::Sender<VisEvent>>;

#[derive(Debug)]
pub struct Visualization {
    normalizer: Normalizer,
    channel: VisualizationChannel,
}

#[derive(Debug)]
pub struct RenderManager {
    pub visualization: Visualization,
    pub renders: [Option<Vec<RenderVoice>>; 2],
    pub store: Option<Vec<Vec<RenderOp>>>,
    pub current_volume: f32,
    pub past_volume: f32,
    render_idx: usize,
    _read_idx: usize,
    kill_channel: KillChannel,
    once: bool,
    paused: bool,
    total_samples_per_loop: usize,
    samples_processed: usize,
}

pub fn render_op_to_normalized_op4d(render_op: &RenderOp, normalizer: &Normalizer) -> Option<Op4D> {
    if render_op.f == 0.0 || render_op.g == (0.0, 0.0) {
        return None;
    };

    let mut op4d = Op4D {
        y: render_op.f,
        z: (render_op.g.0 + render_op.g.1) / 2.0,
        x: render_op.p,
        l: render_op.l,
        t: render_op.t,
        voice: render_op.voice,
        event: render_op.event,
        names: render_op.names.to_vec(),
    };

    op4d.normalize(normalizer);

    Some(op4d)
}

pub struct RenderManagerSettings {
    pub sample_rate: f64,
    pub buffer_size: usize,
}

impl RenderManager {
    pub fn init(
        visualization_channel: VisualizationChannel,
        kill_channel: KillChannel,
        once: bool,
        settings: Option<RenderManagerSettings>,
    ) -> Self {
        if !cfg!(test) {
            if let Some(s) = settings {
                Settings::init(s.sample_rate, s.buffer_size);
            } else {
                Settings::init_default();
            };
        }
        Self {
            visualization: Visualization {
                channel: visualization_channel,
                normalizer: Normalizer::default(),
            },
            renders: [None, None],
            store: None,
            past_volume: 0.8,
            current_volume: 0.8,
            render_idx: 0,
            _read_idx: 0,
            kill_channel,
            once,
            paused: false,
            total_samples_per_loop: 0,
            samples_processed: 0,
        }
    }

    pub fn init_wasm(settings: Option<RenderManagerSettings>) -> Self {
        if !cfg!(test) {
            if let Some(s) = settings {
                Settings::init(s.sample_rate, s.buffer_size);
            } else {
                Settings::init_default();
            };
        }
        Self {
            visualization: Visualization {
                channel: None,
                normalizer: Normalizer::default(),
            },
            renders: [None, None],
            store: None,
            past_volume: 0.8,
            current_volume: 0.8,
            render_idx: 0,
            _read_idx: 0,
            kill_channel: None,
            once: false,
            paused: false,
            total_samples_per_loop: 0,
            samples_processed: 0,
        }
    }

    pub fn kill(&self) -> Result<(), SendError<bool>> {
        if let Some(kc) = &self.kill_channel {
            kc.send(true)?;
            #[cfg(target_os = "linux")]
            std::thread::sleep(std::time::Duration::from_millis(500));
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn play(&mut self) {
        self.paused = false;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn update_volume(&mut self, volume: f32) {
        self.current_volume = f32::powf(volume, 2.0)
    }

    fn ramp_to_current_volume(&mut self, buffer_size: usize) -> Vec<f32> {
        let offset: Vec<f32> = (0..buffer_size * 2)
            .map(|i| {
                let distance = self.current_volume - self.past_volume;
                self.past_volume + (distance * i as f32 / (buffer_size * 2) as f32)
            })
            .collect();

        self.past_volume = self.current_volume;

        offset
    }

    pub fn push_ops_to_store(&mut self, to_store: Vec<Vec<RenderOp>>) {
        if let Some(store) = &mut self.store {
            if store.len() < to_store.len() {
                // Extend the store to match the size of `to_store`
                store.extend((store.len()..to_store.len()).map(|_| Vec::new()));
            }

            store.iter_mut().zip(to_store).for_each(|(voice, ops)| {
                voice.extend(ops.into_iter().map(|mut op| {
                    op.follow = false;
                    op
                }));
            });
        } else {
            self.store = Some(to_store);
        }
    }

    pub fn push_store_to_current_render(&mut self) {
        // Take the store out temporarily
        if let Some(store) = self.store.take() {
            if let Some(current_render) = self.current_render().as_mut() {
                current_render.extend(store.into_iter().map(|ops| RenderVoice::init(&ops.clone())));
            }
            self.store = None
        }
    }

    pub fn read(
        &mut self,
        buffer_size: usize,
        offset: Offset,
    ) -> Option<(StereoWaveform, Vec<f32>, Vec<Vec<RenderOp>>)> {
        if self.paused {
            return None;
        }

        let mut remaining_buffer_size = buffer_size;
        let mut total_rendered_per_batch: Vec<Vec<StereoWaveform>> = Vec::new();
        let mut total_ops: Resizeable2DVec<RenderOp> = Resizeable2DVec::new(1);

        let vtx = self.visualization.channel.clone();
        let normalizer = self.visualization.normalizer;

        while remaining_buffer_size > 0 {
            // Compute next_exists before mutable borrow
            let next_exists = self.exists_next_render();

            // Start mutable borrow scope
            let (samples_processed, render_finished) = {
                let current_render_option = self.current_render();

                match current_render_option {
                    Some(render_voices) => {
                        let mut any_data_rendered = false;
                        let mut rendered_per_voice: Vec<StereoWaveform> = Vec::new();

                        let mut min_samples_processed = remaining_buffer_size;

                        for (i, voice) in render_voices.iter_mut().enumerate() {
                            // TODO: Should return if it looped and reset store
                            // TODO: or should it just push the store to the current render?
                            // TODO: This is getting super complicated...what should I do?
                            // TODO: Maybe factor this out?
                            // TODO: How do I save the state so I can print?
                            // TODO: The store stuff should be behind a feature flag
                            match voice.get_batch(
                                remaining_buffer_size,
                                None,
                                !next_exists && Settings::global().loop_play,
                            ) {
                                Some(mut batch) => {
                                    any_data_rendered = true;
                                    let samples = batch.iter().map(|op| op.samples).sum::<usize>();
                                    min_samples_processed = min_samples_processed.min(samples);

                                    let voice_rendered =
                                        batch.render(&mut voice.oscillator, Some(&offset));
                                    rendered_per_voice.push(voice_rendered);

                                    if let Some(_vtx) = &vtx {
                                        // let b = batch
                                        // .clone()
                                        // .into_iter()
                                        // .map(|mut op| {
                                        // if op.follow {
                                        // op.f = op.f * offset.freq;
                                        // op.g = (
                                        // op.g.0 * offset.gain,
                                        // op.g.1 * offset.gain,
                                        // );
                                        // }
                                        // op
                                        // })
                                        // .collect();
                                        let b: Vec<_> = batch
                                            .iter()
                                            .filter(|op| op.index % 6 == 0)
                                            .cloned()
                                            .map(|mut op| {
                                                if op.follow {
                                                    op.f *= offset.freq;
                                                    op.g = (
                                                        op.g.0 * offset.gain,
                                                        op.g.1 * offset.gain,
                                                    );
                                                }
                                                op
                                            })
                                            .collect();

                                        total_ops.extend_at(i, b);
                                        // ops_per_voice.push(voice_ops);
                                    }
                                }
                                None => {
                                    // Voice has finished
                                }
                            }
                        }

                        if any_data_rendered && min_samples_processed > 0 {
                            // Store the per-voice rendered waveforms for this batch
                            total_rendered_per_batch.push(rendered_per_voice);

                            (min_samples_processed, false)
                        } else if any_data_rendered {
                            // Some data rendered, but min_samples_processed is zero
                            (0, false)
                        } else {
                            // All voices have finished
                            (0, true)
                        }
                    }
                    None => {
                        // No current render
                        (0, true)
                    }
                }
            }; // End of mutable borrow

            if samples_processed > 0 {
                remaining_buffer_size = remaining_buffer_size.saturating_sub(samples_processed);
            }

            if render_finished {
                if self.exists_next_render() {
                    self.inc_render();
                    // self.push_store_to_current_render();
                    continue; // Continue processing with next render
                } else {
                    if self.once {
                        self.kill().expect("Unable to kill");
                    }
                    break; // No more renders, exit loop
                }
            }

            if samples_processed == 0 {
                // No samples processed, break to avoid infinite loop
                break;
            }
        }

        // Now, we have total_rendered_per_batch: Vec<Vec<StereoWaveform>>
        // Each inner Vec<StereoWaveform> corresponds to per-voice waveforms for a batch
        // Now, we need to sum the per-voice waveforms for each batch and append them to build the final combined waveform

        if !total_rendered_per_batch.is_empty() {
            let mut combined_sw = StereoWaveform::new_empty();

            for rendered_per_voice in total_rendered_per_batch {
                let batch_sw = sum_all_waveforms(rendered_per_voice);
                combined_sw.append(batch_sw);
            }

            combined_sw.pad(buffer_size);

            // Visualization
            if let Some(tx) = vtx {
                let ops = total_ops.to_vec_flat();
                let mut opmap: OpMap<Op4D> = OpMap::with_capacity(ops.len());
                ops.iter().for_each(|v| {
                    let name = v.names.last().map_or("nameless", |n| n);

                    let op = render_op_to_normalized_op4d(v, &normalizer);
                    if let Some(o) = op {
                        opmap.insert(name, o);
                    };
                });

                if tx.send(VisEvent::Ops(opmap)).is_err() {
                    info!("Visualization channel closed");
                    std::process::exit(0);
                }
            }

            let ramp = self.ramp_to_current_volume(buffer_size);
            Some((combined_sw, ramp, total_ops.to_vec()))
        } else {
            None
        }
    }

    pub fn inc_render(&mut self) {
        info!("Incrementing render");
        // Update the render index

        // Since self.renders has length 2, we can split it at index 1
        let (first, second) = self.renders.split_at_mut(1);

        let (current_render_option, next_render_option) = if self.render_idx == 0 {
            (&first[0], &mut second[0])
        } else {
            (&second[0], &mut first[0])
        };

        if let (Some(current_voices), Some(next_voices)) =
            (current_render_option.as_ref(), next_render_option.as_mut())
        {
            // Ensure that both renders have the same number of voices
            let min_length = std::cmp::min(current_voices.len(), next_voices.len());
            for i in 0..min_length {
                let current_oscillator = &current_voices[i].oscillator;
                let next_oscillator = &mut next_voices[i].oscillator;

                // Copy the oscillator state
                next_oscillator.copy_state_from(current_oscillator);
            }
        }

        // Reset samples processed for the new render
        self.samples_processed = 0;

        // Update total_samples_per_loop for the new render
        if let Some(next_render) = next_render_option.as_ref() {
            if !next_render.is_empty() {
                self.total_samples_per_loop = next_render[0].ops.iter().map(|op| op.samples).sum();
            }
        }

        // Send visualization reset event if necessary
        if let Some(vtx) = self.visualization.channel.clone() {
            vtx.send(VisEvent::Reset)
                .expect("Couldn't send VisEvent::Reset");
        }

        *self.current_render() = None;
        self.render_idx = (self.render_idx + 1) % 2;
    }

    pub fn current_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[self.render_idx]
    }

    pub fn next_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[(self.render_idx + 1) % 2]
    }

    pub fn current_render_ref(&self) -> &Option<Vec<RenderVoice>> {
        &self.renders[self.render_idx]
    }

    pub fn exists_current_render(&self) -> bool {
        self.renders[(self.render_idx) % 2].is_some()
    }

    pub fn exists_next_render(&self) -> bool {
        self.renders[(self.render_idx + 1) % 2].is_some()
    }

    pub fn push_render(&mut self, render: Vec<RenderVoice>, once: bool) {
        self.once = once;
        *self.next_render() = Some(render);
    }
}

pub fn prepare_render_outside(
    input: InputType<'_>,
    working_path: Option<PathBuf>,
) -> Result<Vec<RenderVoice>, Error> {
    let (nf, basis, mut table) = match input.make(RenderType::NfBasisAndTable, working_path)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => return Err(Error::with_msg("Failed Parse/Render")),
    };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;

    let render_voices = renderables_to_render_voices(renderables);

    Ok(render_voices)
}

#[cfg(test)]
mod render_manager_tests {
    use super::*;
    use weresocool_instrument::renderable::RenderOp;
    use weresocool_shared::helpers::{cmp_f32, cmp_vec_f32};

    #[test]
    fn test_ramp_to_current_value() {
        let mut rm = RenderManager::init(None, None, false, None);
        rm.update_volume(0.9);
        assert!(cmp_f32(rm.current_volume, f32::powf(0.9, 2.0)));
        let ramp = rm.ramp_to_current_volume(2);
        dbg!(&ramp);
        assert!(cmp_vec_f32(
            ramp,
            vec![0.8, 0.8025, 0.80499995, 0.807_499_95]
        ));
    }

    #[test]
    fn test_inc_render() {
        let mut r = RenderManager::init(None, None, false, None);
        r.inc_render();
        assert_eq!(r.render_idx, 1);
        r.inc_render();
        assert_eq!(r.render_idx, 0);
    }

    fn render_voices_mock() -> Vec<RenderVoice> {
        vec![RenderVoice::init(&[RenderOp::init_silent_with_length(1.0)])]
    }

    #[test]
    fn test_push_render() {
        Settings::init_test();
        let mut r = RenderManager::init(None, None, false, None);
        assert_eq!(*r.current_render(), None);
        assert_eq!(*r.next_render(), None);
        r.push_render(render_voices_mock(), false);
        assert_eq!(*r.next_render(), Some(render_voices_mock()));
        assert_eq!(*r.current_render(), None);
        r.push_render(render_voices_mock(), false);
    }
}
