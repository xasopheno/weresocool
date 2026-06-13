use crate::{voice::Voice, SynthOp};
use weresocool_ast::OscType;

pub fn gain_at_index(start: f64, target: f64, index: usize, length: usize) -> f64 {
    let ratio = (index as f64 / length as f64).min(1.0); // Clamp to prevent overshoot
    let distance = target - start;
    start + distance * ratio
}

impl Voice {
    pub fn past_gain_from_op<Op: SynthOp>(&self, op: &Op) -> f64 {
        match self.osc_type {
            OscType::Sine { .. } | OscType::None => match op.oscillator_type() {
                OscType::Sine { .. } | OscType::None => self.current.gain,
                // Drums calibrate their own perceived loudness via KICK_GAIN /
                // SNARE_GAIN / HIHAT_GAIN — skip the /3 sine-vs-noise tilt.
                t if t.is_drum() => self.current.gain,
                _ => self.current.gain / 3.0,
            },
            _ => self.current.gain,
        }
    }

    pub fn current_gain_from_op<Op: SynthOp>(&self, op: &Op) -> f64 {
        let mut gain = if op.frequency() > 20.0 {
            (op.gain_left(), op.gain_right())
        } else {
            (0., 0.)
        };

        gain = match op.oscillator_type() {
            OscType::Sine { .. } | OscType::None => gain,
            t if t.is_drum() => gain,
            _ => (gain.0 / 3.0, gain.1 / 3.0),
        };

        match self.index {
            0 => gain.0,
            _ => gain.1,
        }
    }

    pub fn silence_next<Op: SynthOp>(&self, op: &Op) -> bool {
        match self.index {
            0 => op.next_left_silent(),
            1 => op.next_right_silent(),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn silence_now(&self) -> bool {
        self.current.silent()
    }

    #[inline]
    pub fn silence_to_sound(&self) -> bool {
        self.past.silent() && !self.current.silent()
    }

    #[inline]
    pub fn sound_to_silence(&self) -> bool {
        !self.past.silent() && self.current.silent()
    }
}

// Tests moved to weresocool_instrument where RenderOp is defined
// Generic tests for SynthOp trait can be added here later
