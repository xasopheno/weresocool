use crate::instrument::loudness::loudness_normalization;
use crate::renderable::{Offset, RenderOp};
use rand::{thread_rng, Rng};
use socool_ast::{OscType, ASR};

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub mic_past: VoiceState,
    pub mic_current: VoiceState,
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
            mic_past: VoiceState::init(),
            mic_current: VoiceState::init(),
            phase: 0.0,
            osc_type: OscType::Sine,
            attack: 44_100,
            decay: 44_100,
            portamento_index: 0,
            asr: ASR::Long,
        }
    }
    pub fn generate_waveform(&mut self, op: &RenderOp, offset: &Offset) -> Vec<f64> {
        //println!("freq {}, gain {}", offset.freq, offset.gain);
        let mut buffer: Vec<f64> = vec![0.0; op.samples];
        //println!("{}, {}", self.current.gain, self.mic_current.gain);

        let p_delta = self.calculate_portamento_delta(
            op.portamento,
            self.mic_past.frequency,
            self.mic_current.frequency,
        );
        let silence_now = self.current.gain == 0.0 || self.current.frequency == 0.0;

        let silent_next = match self.index {
            0 => op.next_l_silent,
            1 => op.next_r_silent,
            _ => unimplemented!(),
        };

        let rand = thread_rng().gen_range(0.5, 1.0);
        //let rand = 1.0;
        let op_gain = self.calculate_gain(
            self.past.gain,
            self.current.gain,
            self.attack,
            self.decay,
            silent_next,
            silence_now,
            op.index + op.samples,
            op.total_samples,
        );

        self.mic_past.gain = self.mic_current.gain;
        self.mic_current.gain = op_gain * rand;

        let mut f = 0.0;
        for (index, sample) in buffer.iter_mut().enumerate() {
            let frequency = self.calculate_frequency(
                index,
                self.portamento_index,
                op.portamento,
                p_delta,
                self.mic_past.frequency,
                self.mic_current.frequency,
                self.sound_to_silence(),
                self.silence_to_sound(),
            );
            f = frequency;

            let gain = self.mic_past.gain
                + (self.mic_current.gain - self.mic_past.gain) * index as f64 / op.samples as f64;
            //let gain = self.calculate_gain(
            //self.past.gain,
            //self.current.gain,
            //self.attack,
            //self.decay,
            //silent_next,
            //silence_now,
            //op.index + index,
            //op.total_samples,
            //);

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
        self.mic_current.frequency = f;

        buffer
    }

    pub fn update(&mut self, op: &RenderOp, offset: &Offset) {
        self.portamento_index = 0;

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
        self.mic_past.frequency = self.mic_current.frequency;
        self.mic_current.frequency = if self.sound_to_silence() {
            self.past.frequency * offset.freq
        //self.past.frequency * thread_rng().gen_range(0.9, 1.1)
        } else {
            self.current.frequency * offset.freq
            //self.current.frequency * thread_rng().gen_range(0.9, 1.1)
        }
    }
    fn calculate_frequency(
        &self,
        index: usize,
        portamento_index: usize,
        portamento_length: usize,
        p_delta: f64,
        start: f64,
        target: f64,
        sound_to_silence: bool,
        silence_to_sound: bool,
    ) -> f64 {
        if sound_to_silence {
            return start;
        } else if portamento_index < portamento_length && !silence_to_sound && !sound_to_silence {
            return start + index as f64 * p_delta;
        } else {
            return target;
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

    pub fn mic_silence_to_sound(&self) -> bool {
        unimplemented!()
    }
    pub fn mic_sound_to_silence(&self) -> bool {
        unimplemented!()
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.silent() && !self.current.silent()
        //self.mic_past.frequency == 0.0 && self.mic_current.frequency != 0.0
    }

    pub fn sound_to_silence(&self) -> bool {
        !self.past.silent() && self.current.silent()
        //self.mic_past.frequency != 0.0 && self.mic_current.frequency == 0.0
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
