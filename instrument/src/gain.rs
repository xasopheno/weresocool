use crate::{renderable::RenderOp, voice::Voice};
use weresocool_ast::OscType;

pub fn gain_at_index(start: f64, target: f64, index: usize, length: usize) -> f64 {
    let distance = target - start;
    start + (distance * index as f64 / length as f64)
}

impl Voice {
    pub fn past_gain_from_op(&self, op: &RenderOp) -> f64 {
        match self.osc_type {
            OscType::Sine { .. } => match op.osc_type {
                OscType::Sine { .. } => self.current.gain,
                _ => self.current.gain / 3.0,
            },
            _ => self.current.gain,
        }
    }

    pub fn current_gain_from_op(&self, op: &RenderOp) -> f64 {
        let mut gain = if op.f > 20.0 { op.g } else { (0., 0.) };

        gain = match op.osc_type {
            OscType::Sine { .. } => gain,
            _ => (gain.0 / 3.0, gain.1 / 3.0),
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

#[cfg(test)]
mod tests {
    use super::*;
    use weresocool_shared::helpers::cmp_f64;
    #[test]
    fn test_get_current_gain_from_op() {
        let v = Voice::init(0);
        let mut op = RenderOp::init_fglp(200.0, (0.9, 0.9), 1.0, 0.0);
        op.osc_type = OscType::Noise;

        let result = v.current_gain_from_op(&op);
        assert!(cmp_f64(result, 0.3));

        op.osc_type = OscType::Sine { pow: None };
        let result = v.current_gain_from_op(&op);
        assert!(cmp_f64(result, 0.9));

        op.f = 0.0;
        let result = v.current_gain_from_op(&op);
        assert!(cmp_f64(result, 0.0));
    }

    #[test]
    fn test_get_past_gain_from_op() {
        let mut v = Voice::init(0);
        let mut op = RenderOp::init_fglp(200.0, (1.0, 1.0), 1.0, 0.0);
        op.osc_type = OscType::Noise;

        v.current.gain = 0.9;

        let result = v.past_gain_from_op(&op);

        assert!(cmp_f64(result, 0.3));
    }

    #[test]
    fn test_silence_next() {
        let mut op = RenderOp::init_silent_with_length(1.0);
        op.next_r_silent = false;
        op.next_l_silent = true;
        let v1 = Voice::init(0);
        let result = v1.silence_next(&op);
        assert!(result);

        let v2 = Voice::init(1);
        let result = v2.silence_next(&op);
        assert!(!result);
    }

    #[test]
    fn test_gain_from_index() {
        let mut g = gain_at_index(0.0, 1.0, 5, 10);
        assert!(cmp_f64(g, 0.5));

        g = gain_at_index(1.0, 0.0, 5, 10);
        assert!(cmp_f64(g, 0.5));

        g = gain_at_index(0.9, 1.0, 2, 10);
        assert!(cmp_f64(g, 0.92));
    }
    #[test]
    fn test_silence_now() {
        let mut v = Voice::init(0);
        v.current.frequency = 0.0;
        v.current.gain = 1.0;
        assert!(v.silence_now());
        v.current.frequency = 19.0;
        v.current.gain = 1.0;
        assert!(v.silence_now());

        v.current.frequency = 100.0;
        v.current.gain = 0.0;
        assert!(v.silence_now());

        v.current.frequency = 20.0;
        v.current.gain = 1.0;
        assert!(!v.silence_now());
    }
    #[test]
    fn test_silence_to_sound() {
        let mut v = Voice::init(0);
        v.past.frequency = 0.0;
        v.current.frequency = 100.0;
        v.past.gain = 1.0;
        v.current.gain = 1.0;
        assert!(v.silence_to_sound());
        assert!(!v.sound_to_silence());

        v.past.frequency = 0.0;
        v.current.frequency = 100.0;
        v.past.gain = 0.0;
        v.current.gain = 1.0;
        assert!(v.silence_to_sound());
        assert!(!v.sound_to_silence());

        v.past.frequency = 0.0;
        v.current.frequency = 100.0;
        v.past.gain = 0.0;
        v.current.gain = 0.0;
        assert!(!v.silence_to_sound());
        assert!(!v.sound_to_silence());
    }
}
