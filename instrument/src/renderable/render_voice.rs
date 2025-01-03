use crate::renderable::{Offset, RenderOp, Renderable};
use crate::{Oscillator, StereoWaveform};

#[cfg(feature = "app")]
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
                osc_type: current_op.osc_type.clone(),
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
                osc_type: current_op.osc_type.clone(),
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
    renderables
        .iter()
        .map(|voice| RenderVoice::init(voice))
        .collect::<Vec<RenderVoice>>()
}
