use crate::{instrument::voice::Voice, renderable::RenderOp};
use socool_ast::OscType;

pub fn gain_at_index(start: f64, target: f64, index: usize, length: usize) -> f64 {
    let distance = target - start;
    start + (distance * index as f64 / length as f64)
}

impl Voice {
    pub fn past_gain_from_op(&self, op: &RenderOp) -> f64 {
        if self.osc_type == OscType::Sine && op.osc_type != OscType::Sine {
            self.current.gain / 3.0
        } else {
            self.current.gain
        }
    }

    pub fn current_gain_from_op(&self, op: &RenderOp) -> f64 {
        let mut gain = if op.f != 0.0 { op.g } else { (0., 0.) };

        gain = if op.osc_type == OscType::Sine {
            gain
        } else {
            (gain.0 / 3.0, gain.1 / 3.0)
        };

        match self.index {
            0 => gain.0,
            _ => gain.1,
        }
    }

    pub fn silence_next(&self, op: &RenderOp) -> bool {
        match self.index {
            0 => op.next_l_silent,
            1 => op.next_r_silent,
            _ => unimplemented!(),
        }
    }

    pub fn silence_now(&self) -> bool {
        self.current.silent()
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.silent() && !self.current.silent()
    }

    pub fn sound_to_silence(&self) -> bool {
        !self.past.silent() && self.current.silent()
    }
}
