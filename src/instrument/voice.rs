use crate::{
    instrument::{asr::gain_at_index, loudness::loudness_normalization},
    renderable::{Offset, RenderOp},
};
use socool_ast::{OscType, ASR};

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub offset_past: VoiceState,
    pub offset_current: VoiceState,
    pub phase: f64,
    pub osc_type: OscType,
    pub attack: usize,
    pub decay: usize,
    pub asr: ASR,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleInfo {
    pub gain: f64,
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

pub struct GainInput {
    pub past_gain: f64,
    pub current_gain: f64,
    pub attack_length: usize,
    pub decay_length: usize,
    pub silent_next: bool,
    pub silence_now: bool,
    pub index: usize,
    pub total_samples: usize,
}

struct FrequencyInput {
    index: usize,
    portamento_length: usize,
    p_delta: f64,
    start: f64,
    target: f64,
    sound_to_silence: bool,
    silence_to_sound: bool,
}

impl Voice {
    pub fn init(index: usize) -> Voice {
        Voice {
            index,
            past: VoiceState::init(),
            current: VoiceState::init(),
            offset_past: VoiceState::init(),
            offset_current: VoiceState::init(),
            phase: 0.0,
            osc_type: OscType::Sine,
            attack: 44_100,
            decay: 44_100,
            asr: ASR::Long,
        }
    }
    pub fn generate_waveform(&mut self, op: &RenderOp, offset: &Offset) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![0.0; op.samples];

        let p_delta = self.calculate_portamento_delta(
            op.portamento,
            self.offset_past.frequency,
            self.offset_current.frequency,
        );

        let silence_now = self.current.gain == 0.0 || self.current.frequency == 0.0;

        let silent_next = match self.index {
            0 => op.next_l_silent,
            1 => op.next_r_silent,
            _ => unimplemented!(),
        };

        let op_gain = self.calculate_gain(GainInput {
            past_gain: self.past.gain,
            current_gain: self.current.gain,
            attack_length: self.attack,
            decay_length: self.decay,
            silent_next,
            silence_now,
            index: op.index + op.samples,
            total_samples: op.total_samples,
        }) * loudness_normalization(self.offset_current.frequency);

        for (index, sample) in buffer.iter_mut().enumerate() {
            let frequency = self.calculate_frequency(FrequencyInput {
                index,
                portamento_length: op.portamento,
                p_delta,
                start: self.offset_past.frequency,
                target: self.offset_current.frequency,
                sound_to_silence: self.sound_to_silence(),
                silence_to_sound: self.silence_to_sound(),
            });

            let gain = gain_at_index(
                self.offset_past.gain,
                op_gain * offset.gain,
                index,
                if op.samples > 250 { op.samples } else { 250 },
            );

            let info = SampleInfo { gain, frequency };

            let new_sample = match self.osc_type {
                OscType::Sine => self.generate_sine_sample(info),
                OscType::Square => self.generate_square_sample(info),
                OscType::Noise => self.generate_random_sample(info),
            };

            if index == op.samples - 1 {
                self.offset_current.frequency = frequency;
                self.offset_current.gain = gain;
            };
            *sample += new_sample
        }

        buffer
    }

    pub fn update(&mut self, op: &RenderOp, offset: &Offset) {
        if op.index == 0 {
            self.past.frequency = self.current.frequency;
            self.current.frequency = op.f;

            self.past.gain = self.past_gain_from_op(op);
            self.current.gain = self.current_gain_from_op(op);

            self.osc_type = op.osc_type;

            self.attack = op.attack.trunc() as usize;
            self.decay = op.decay.trunc() as usize;

            self.asr = op.asr;
        };
        self.offset_past.gain = self.offset_current.gain;
        self.offset_past.frequency = self.offset_current.frequency;

        self.offset_current.frequency = if self.sound_to_silence() {
            self.past.frequency * offset.freq
        } else {
            self.current.frequency * offset.freq
        }
    }
    fn calculate_frequency(
        &self,
        fi: FrequencyInput, //index: usize,
                            //portamento_length: usize,
                            //p_delta: f64,
                            //start: f64,
                            //target: f64,
                            //sound_to_silence: bool,
                            //silence_to_sound: bool
    ) -> f64 {
        if fi.sound_to_silence {
            fi.start
        } else if fi.index < fi.portamento_length && !fi.silence_to_sound && !fi.sound_to_silence {
            fi.start + fi.index as f64 * fi.p_delta
        } else {
            fi.target
        }
    }

    fn past_gain_from_op(&self, op: &RenderOp) -> f64 {
        if self.osc_type == OscType::Sine && op.osc_type != OscType::Sine {
            self.current.gain / 3.0
        } else {
            self.current.gain
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
            0 => gain.0,
            _ => gain.1,
        }
    }

    pub fn mic_silence_to_sound(&self) -> bool {
        unimplemented!()
    }
    pub fn mic_sound_to_silence(&self) -> bool {
        unimplemented!()
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.silent() && !self.current.silent()
    }

    pub fn sound_to_silence(&self) -> bool {
        !self.past.silent() && self.current.silent()
    }

    pub fn calculate_portamento_delta(
        &self,
        portamento_length: usize,
        start: f64,
        target: f64,
    ) -> f64 {
        (target - start) / (portamento_length as f64)
    }
}
