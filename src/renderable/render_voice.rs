use crate::{
    instrument::{Oscillator, StereoWaveform},
    renderable::{Offset, RenderOp, Renderable},
    settings::default_settings,
};

#[derive(Debug, Clone)]
pub struct RenderVoice {
    pub sample_index: usize,
    pub op_index: usize,
    pub ops: Vec<RenderOp>,
    pub oscillator: Oscillator,
}

impl RenderVoice {
    pub fn init(ops: &Vec<RenderOp>) -> RenderVoice {
        RenderVoice {
            sample_index: 0,
            op_index: 0,
            ops: ops.to_vec(),
            oscillator: Oscillator::init(&default_settings()),
        }
    }

    /// Recursive function to prepare a batch of RenderOps for rendering
    /// Initially pass in None as result
    /// ```
    /// # use weresocool::renderable::{RenderOp, RenderVoice};
    /// let mut voice = RenderVoice::init(&vec![RenderOp::init_silent_with_length(1.0)]);
    /// let batch = voice.get_batch(1024, None);
    /// ```
    pub fn get_batch(
        &mut self,
        samples_left_in_batch: usize,
        result: Option<Vec<RenderOp>>,
    ) -> Vec<RenderOp> {
        let mut result: Vec<RenderOp> = match result {
            Some(result) => result.to_vec(),
            None => vec![],
        };

        if false {
            if self.op_index >= self.ops.len() {
                self.op_index = 0;
            }
        }

        if self.op_index >= self.ops.len() {
            let mut op = RenderOp::init_silent_with_length(1.0);
            result.push(op);
            return result;
        }

        let current_op = &self.ops[self.op_index];

        if (current_op.samples - self.sample_index) > samples_left_in_batch {
            result.push(RenderOp {
                samples: samples_left_in_batch,
                index: self.sample_index,
                ..*current_op
            });
            self.sample_index += samples_left_in_batch;
        } else {
            let n_samples = current_op.samples - self.sample_index;
            result.push(RenderOp {
                samples: n_samples,
                index: self.sample_index,
                ..*current_op
            });

            self.op_index += 1;

            self.sample_index = 0;

            return self.get_batch(samples_left_in_batch - n_samples, Some(result));
        }

        result
    }

    pub fn render_batch(&mut self, n_samples: usize, offset: Option<&Offset>) -> StereoWaveform {
        let mut batch = self.get_batch(n_samples, None);

        batch.render(&mut self.oscillator, offset)
    }
}

pub fn renderables_to_render_voices(renderables: Vec<Vec<RenderOp>>) -> Vec<RenderVoice> {
    renderables
        .iter()
        .map(|voice| RenderVoice::init(voice))
        .collect::<Vec<RenderVoice>>()
}
