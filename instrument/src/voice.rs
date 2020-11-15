use crate::{
    renderable::{Offset, RenderOp},
    {gain::gain_at_index, loudness::loudness_normalization},
};

use reverb::Reverb;
use weresocool_ast::{OscType, ASR};
use weresocool_shared::{default_settings, Settings};

const SETTINGS: Settings = default_settings();

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub reverb: Reverb,
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
    pub frequency: f64,
    pub gain: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceState {
    pub frequency: f64,
    pub gain: f64,
}
impl VoiceState {
    pub const fn init() -> Self {
        Self {
            frequency: 0.0,
            gain: 0.0,
        }
    }
    pub fn silent(&self) -> bool {
        self.frequency < SETTINGS.min_freq || self.gain == 0.0
    }
}

impl Voice {
    pub fn init(index: usize) -> Self {
        Self {
            reverb: Reverb::new(),
            index,
            past: VoiceState::init(),
            current: VoiceState::init(),
            offset_past: VoiceState::init(),
            offset_current: VoiceState::init(),
            phase: 0.0,
            osc_type: OscType::Sine { pow: None },
            attack: SETTINGS.sample_rate as usize,
            decay: SETTINGS.sample_rate as usize,
            asr: ASR::Long,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.reverb.is_empty()
    }

    pub fn generate_waveform(&mut self, op: &RenderOp, offset: &Offset) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![0.0; op.samples];

        let p_delta = self.calculate_portamento_delta(
            op.portamento,
            self.offset_past.frequency,
            self.offset_current.frequency,
        );

        let op_gain = self.calculate_op_gain(
            self.silence_now(),
            self.silence_next(op),
            op.index + op.samples,
            op.total_samples,
        ) * loudness_normalization(self.offset_current.frequency);

        self.reverb.update(op.reverb as f32);

        for (index, sample) in buffer.iter_mut().enumerate() {
            let frequency = self.calculate_frequency(
                index,
                op.portamento,
                p_delta,
                self.offset_past.frequency,
                self.offset_current.frequency,
            );
            let gain = gain_at_index(
                self.offset_past.gain,
                op_gain * offset.gain,
                index,
                if op.samples > 250 { op.samples } else { 250 },
            );

            let info = SampleInfo { gain, frequency };

            let mut new_sample = match self.osc_type {
                OscType::Sine { pow } => self.generate_sine_sample(info, pow),
                OscType::Square => self.generate_square_sample(info),
                OscType::Noise => self.generate_random_sample(info),
            };

            if op.reverb > 0.0 {
                new_sample = self.reverb.calc_sample(new_sample, gain);
            }

            if index == op.samples - 1 {
                self.offset_current.frequency = frequency;
                self.offset_current.gain = gain;
            };

            *sample += new_sample
        }

        // if op_gain == 0.0 && self.past.gain != 0.0 {
        // self.reverb.reset();
        // }

        buffer.to_vec()
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
}
