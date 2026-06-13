use crate::renderable::{Offset, RenderOp, Renderable};
use crate::{Oscillator, StereoWaveform};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderVoice {
    pub sample_index: usize,
    pub op_index: usize,
    pub ops: Vec<RenderOp>,
    pub oscillator: Oscillator,
}

impl Default for RenderVoice {
    fn default() -> Self {
        Self {
            sample_index: 0,
            op_index: 0,
            ops: vec![],
            oscillator: Oscillator::init(),
        }
    }
}

impl RenderVoice {
    pub fn init(ops: &[RenderOp]) -> Self {
        Self {
            sample_index: 0,
            op_index: 0,
            ops: ops.to_owned(),
            oscillator: Oscillator::init(),
        }
    }

    /// Move this voice's cursor to the absolute sample position
    /// `target_sample` from the start of the voice's op list, resetting
    /// oscillator state to silence any history-dependent artifacts.
    ///
    /// Used by `AudioEngine::seek_to_sample` for scrub. The oscillator
    /// reset will produce an audible click — the AudioEngine layer
    /// masks it with a short master gain ramp so we don't have to do it
    /// here.
    ///
    /// If `target_sample` lies past the end of this voice's timeline,
    /// the cursor parks at the end (the voice will produce no further
    /// samples, which is the correct steady state).
    pub fn seek(&mut self, target_sample: usize) {
        // Reset oscillator state. `Oscillator::init` is what `RenderVoice`
        // itself uses on construction; reusing it keeps the "fresh
        // voice" semantics identical.
        self.oscillator = Oscillator::init();

        let mut cumulative: usize = 0;
        for (i, op) in self.ops.iter().enumerate() {
            let op_end = cumulative + op.samples;
            if target_sample < op_end {
                self.op_index = i;
                self.sample_index = target_sample - cumulative;
                return;
            }
            cumulative = op_end;
        }
        // Past the end of the timeline: park at the last op, fully consumed.
        // `get_batch` returns `None` from here, which the engine treats as
        // "voice finished" — the right behaviour for "scrub past the end."
        self.op_index = self.ops.len();
        self.sample_index = 0;
    }

    /// Recursive function to prepare a batch of RenderOps for rendering
    /// Initially pass in None as result
    pub fn get_batch(
        &mut self,
        samples_left_in_batch: usize,
        result: Option<Vec<RenderOp>>,
        loop_play: bool,
    ) -> Option<Vec<RenderOp>> {
        let mut result = result.unwrap_or_default();

        if loop_play && self.op_index >= self.ops.len() {
            self.op_index = 0;
        }

        if self.op_index >= self.ops.len() {
            return if result.is_empty() {
                None
            } else {
                Some(result)
            };
        }

        let current_op = &self.ops[self.op_index];

        if (current_op.samples - self.sample_index) > samples_left_in_batch {
            result.push(RenderOp {
                samples: samples_left_in_batch,
                index: self.sample_index,
                names: current_op.names.clone(),
                filters: current_op.filters.clone(),
                distortions: current_op.distortions.clone(),
                osc_type: current_op.osc_type.clone(),
                follows: current_op.follows.clone(),
                colors: current_op.colors.clone(),
                wgsl: current_op.wgsl.clone(),
                midi: current_op.midi.clone(),
                ..*current_op
            });
            self.sample_index += samples_left_in_batch;
        } else {
            let n_samples = current_op.samples - self.sample_index;
            result.push(RenderOp {
                samples: n_samples,
                index: self.sample_index,
                names: current_op.names.clone(),
                filters: current_op.filters.clone(),
                distortions: current_op.distortions.clone(),
                osc_type: current_op.osc_type.clone(),
                follows: current_op.follows.clone(),
                colors: current_op.colors.clone(),
                wgsl: current_op.wgsl.clone(),
                midi: current_op.midi.clone(),
                ..*current_op
            });

            self.op_index += 1;

            self.sample_index = 0;

            return self.get_batch(samples_left_in_batch - n_samples, Some(result), loop_play);
        }

        Some(result)
    }

    pub fn render_batch(
        &mut self,
        n_samples: usize,
        offset: Option<&Offset>,
    ) -> Option<StereoWaveform> {
        let batch = self.get_batch(n_samples, None, false);

        batch.map(|mut b| b.render(&mut self.oscillator, offset))
    }
}

pub fn renderables_to_render_voices(renderables: Vec<Vec<RenderOp>>) -> Vec<RenderVoice> {
    // `into_iter` so each voice's `Vec<RenderOp>` moves into the resulting
    // `RenderVoice` instead of being cloned. The previous `.iter()` + `RenderVoice::init`
    // path deep-cloned every `RenderOp` (~178 MB on drum_sounds.socool).
    renderables
        .into_iter()
        .map(|ops| RenderVoice {
            sample_index: 0,
            op_index: 0,
            ops,
            oscillator: Oscillator::init(),
        })
        .collect()
}
