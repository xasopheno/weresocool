use crate::instrument::{asr::calculate_gain, loudness::loudness_normalization};
use socool_ast::OscType;
use std::f64::consts::PI;

fn tau() -> f64 {
    PI * 2.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub phase: f64,
    pub osc_type: OscType,
    pub attack: usize,
    pub decay: usize,
    pub decay_length: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleInfo {
    pub index: usize,
    pub p_delta: f64,
    pub gain: f64,
    pub portamento_length: usize,
    pub factor: f64,
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
}

#[derive(Clone, Debug)]
pub struct VoiceUpdate {
    pub frequency: f64,
    pub gain: f64,
    pub osc_type: OscType,
    pub silence_next: bool,
    pub attack: f64,
    pub decay: f64,
    pub decay_type: usize,
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
            decay_length: 2,
        }
    }
    pub fn generate_waveform(
        &mut self,
        buffer: &mut Vec<f64>,
        portamento_length: usize,
        starting_index: usize,
        total_samples: usize,
        silent_next: bool,
    ) {
        let factor: f64 = tau() / 44_100.0;
        let p_delta = self.calculate_portamento_delta(portamento_length);
        let silence_now = self.current.gain == 0.0 || self.current.frequency == 0.0;
        for (index, sample) in buffer.iter_mut().enumerate() {
            let gain = calculate_gain(
                self.past.gain,
                self.current.gain,
                silent_next,
                silence_now,
                starting_index + index,
                self.attack,
                self.decay,
                total_samples,
                self.decay_length,
            );
            let info = SampleInfo {
                index: index + starting_index,
                p_delta,
                gain,
                portamento_length,
                factor,
            };
            let new_sample = match self.osc_type {
                OscType::Sine => self.generate_sine_sample(info),
                OscType::Square => self.generate_square_sample(info),
                OscType::Noise => self.generate_random_sample(info),
            };

            *sample += new_sample
        }
    }

    pub fn update(&mut self, info: VoiceUpdate) {
        self.past.frequency = self.current.frequency;
        self.current.frequency = info.frequency;

        self.past.gain = self.current.gain;
        self.current.gain = info.gain * loudness_normalization(info.frequency);

        self.osc_type = info.osc_type;

        self.attack = info.attack.trunc() as usize;
        self.decay = info.decay.trunc() as usize;

        self.decay_length = info.decay_type;
    }

    pub fn silent(&self) -> bool {
        self.current.frequency == 0.0 || self.current.gain == 0.0
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.frequency == 0.0 && self.current.frequency != 0.0
    }

    pub fn sound_to_silence(&self) -> bool {
        self.past.frequency != 0.0 && self.current.frequency == 0.0
    }

    pub fn calculate_portamento_delta(&self, portamento_length: usize) -> f64 {
        (self.current.frequency - self.past.frequency) / (portamento_length as f64)
    }

    pub fn is_short(&self, buffer_len: usize) -> bool {
        buffer_len <= self.attack + self.decay
    }

    pub fn calculate_attack(
        &self,
        distance: f64,
        attack_index: usize,
        attack_length: usize,
    ) -> f64 {
        self.past.gain + (distance * attack_index as f64 / attack_length as f64)
    }

    pub fn calculate_decay(&self, distance: f64, decay_index: usize, decay_length: usize) -> f64 {
        distance * decay_index as f64 / decay_length as f64
    }
}
