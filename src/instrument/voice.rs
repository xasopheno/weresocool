use crate::instrument::loudness::loudness_normalization;
use crate::renderable::{Offset, RenderOp};
use socool_ast::{OscType, ASR};

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub phase: f64,
    pub osc_type: OscType,
    pub attack: usize,
    pub decay: usize,
    pub portamento_index: usize,
    pub asr: ASR,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleInfo {
    pub index: usize,
    pub gain: f64,
    pub portamento_length: usize,
    pub frequency: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceState {
    pub frequency: f64,
    pub gain: f64,
}

impl VoiceState {
    fn init() -> VoiceState {
        VoiceState {
            frequency: 0.0,
            gain: 0.0,
        }
    }
    fn silent(&self) -> bool {
        self.frequency < 20.0 || self.gain == 0.0
    }
}

impl Voice {
    pub fn init(index: usize) -> Voice {
        Voice {
            index,
            past: VoiceState::init(),
            current: VoiceState::init(),
            phase: 0.0,
            osc_type: OscType::Sine,
            attack: 44_100,
            decay: 44_100,
            portamento_index: 0,
            asr: ASR::Long,
        }
    }
    pub fn generate_waveform(&mut self, op: &RenderOp, _offset: &Offset) -> Vec<f64> {
        //println!("freq {}, gain {}", offset.freq, offset.gain);
        let mut buffer: Vec<f64> = vec![0.0; op.samples];

        let p_delta = self.calculate_portamento_delta(op.portamento);
        let silence_now = self.current.gain == 0.0 || self.current.frequency == 0.0;

        let silent_next = match self.index {
            0 => op.next_l_silent,
            _ => op.next_r_silent,
        };

        for (index, sample) in buffer.iter_mut().enumerate() {
            let frequency = self.calculate_frequency(index + op.index, op.portamento, p_delta);

            let gain = self.calculate_gain(
                self.past.gain,
                self.current.gain,
                self.attack,
                self.decay,
                silent_next,
                silence_now,
                op.index + index,
                op.total_samples,
            );

            let info = SampleInfo {
                portamento_length: op.portamento,
                index: op.index + index,
                gain,
                frequency,
            };
            let new_sample = match self.osc_type {
                OscType::Sine => self.generate_sine_sample(info),
                OscType::Square => self.generate_square_sample(info),
                OscType::Noise => self.generate_random_sample(info),
            };
            self.portamento_index += 1;

            *sample += new_sample
        }

        buffer
    }

    pub fn update(&mut self, op: &RenderOp) {
        self.portamento_index = 0;

        self.past.frequency = self.current.frequency;
        self.current.frequency = op.f;

        self.past.gain = self.past_gain_from_op(op);
        self.current.gain = self.current_gain_from_op(op);

        self.osc_type = op.osc_type;

        self.attack = op.attack.trunc() as usize;
        self.decay = op.decay.trunc() as usize;

        self.asr = op.asr;
    }

    fn calculate_frequency(&self, index: usize, portamento: usize, p_delta: f64) -> f64 {
        if self.sound_to_silence() {
            return self.past.frequency;
        } else if self.portamento_index < portamento
            && !self.silence_to_sound()
            && !self.sound_to_silence()
        {
            return self.past.frequency + index as f64 * p_delta;
        } else {
            return self.current.frequency;
        };
    }
    fn past_gain_from_op(&self, op: &RenderOp) -> f64 {
        if self.osc_type == OscType::Sine && op.osc_type != OscType::Sine {
            return self.current.gain / 3.0;
        } else {
            return self.current.gain;
        }
    }

    fn current_gain_from_op(&self, op: &RenderOp) -> f64 {
        let mut gain = if op.f != 0.0 { op.g } else { (0., 0.) };

        gain = if op.osc_type == OscType::Sine {
            gain
        } else {
            (gain.0 / 3.0, gain.1 / 3.0)
        };

        match self.index {
            0 => return gain.0 * loudness_normalization(op.f),
            _ => return gain.1 * loudness_normalization(op.f),
        };
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.silent() && !self.current.silent()
    }

    pub fn sound_to_silence(&self) -> bool {
        !self.past.silent() && self.current.silent()
    }

    pub fn calculate_portamento_delta(&self, portamento_length: usize) -> f64 {
        // TODO: shouldn't take self. should be functional
        // Also should be moved to portamento.rs.
        // gain.rs and portamento.rs - not asr...
        (self.current.frequency - self.past.frequency) / (portamento_length as f64)
    }
}
