use crate::generation::Normalizer;
use crate::{generation::sum_all_waveforms, generation::Op4D};
use opmap::OpMap;
use std::sync::mpsc::SendError;
use std::sync::mpsc::Sender;
use weresocool_instrument::renderable::{RenderOp, RenderVoice, Renderable};
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
pub struct ServerRenderManager {
    pub visualization: Visualization,
    pub voice: RenderVoice,
    pub current_volume: f32,
    pub past_volume: f32,
    _read_idx: usize,
    kill_channel: KillChannel,
    paused: bool,
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

pub struct ServerRenderManagerSettings {
    pub sample_rate: f64,
    pub buffer_size: usize,
}

impl ServerRenderManager {
    pub fn init(
        visualization_channel: VisualizationChannel,
        kill_channel: KillChannel,
        settings: Option<ServerRenderManagerSettings>,
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
            voice: RenderVoice::init(&[]),
            past_volume: 0.8,
            current_volume: 0.8,
            _read_idx: 0,
            kill_channel,
            paused: false,
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

    pub fn push(&mut self, mut op: RenderOp) {
        op.f *= 2.0;
        self.voice.ops.push(op);
    }

    pub fn read(&mut self, buffer_size: usize) -> Option<(StereoWaveform, Vec<f32>)> {
        if self.paused {
            return None;
        };

        let vtx = self.visualization.channel.clone();
        let normalizer = self.visualization.normalizer;

        let ops = self
            .voice
            .get_batch(Settings::global().buffer_size, None, false);
        let result = match ops {
            Some(mut batch) => Some((
                if vtx.is_some() {
                    batch
                        .iter()
                        .filter(|op| op.index == 0)
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                },
                batch.render(&mut self.voice.oscillator, None),
            )),
            None => None,
        };

        let (ops, rendered): (Vec<_>, Vec<_>) = result.into_iter().unzip();

        if let Some(tx) = vtx {
            let mut opmap: OpMap<Op4D> = OpMap::default();

            ops.iter().flatten().for_each(|v| {
                let name = v.names.last().map_or("nameless", |n| n);

                let op = render_op_to_normalized_op4d(v, &normalizer);
                if let Some(o) = op {
                    opmap.insert(name, o);
                };
            });
            tx.send(VisEvent::Ops(opmap)).unwrap();
        }

        if !rendered.is_empty() {
            let mut sw: StereoWaveform = sum_all_waveforms(rendered);

            sw.pad(buffer_size);

            let ramp = self.ramp_to_current_volume(buffer_size);
            Some((sw, ramp))
        } else {
            None
        }
    }
}
