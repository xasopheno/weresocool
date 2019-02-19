extern crate socool_ast;
use instrument::loudness::loudness_normalization;
use socool_ast::ast::OscType;

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub phase: f64,
    pub osc_type: OscType,
    pub attack: usize,
    pub decay: usize,
    pub asr: ASR,
    pub counter: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ASR {
    ASR,
    AS,
    S,
    SR,
    Silence
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleInfo {
    pub index: usize,
    pub p_delta: f64,
    pub g_delta: f64,
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

impl Voice {
    pub fn init(index: usize) -> Voice {
        Voice {
            index,
            past: VoiceState::init(),
            current: VoiceState::init(),
            phase: 0.0,
            osc_type: OscType::Sine,
            attack: 2000,
            decay: 2000,
            asr: ASR::Silence,
            counter: 0,
        }
    }
    pub fn generate_waveform(
        &mut self,
        buffer: &mut Vec<f64>,
        portamento_length: usize,
        factor: f64,
    ) {
        let p_delta = self.calculate_portamento_delta(portamento_length);
//        let g_delta = self.calculate_gain_delta(buffer.len(), silence_next);

        let buffer_len = buffer.len();

        for (index, sample) in buffer.iter_mut().enumerate() {
            let info = SampleInfo {
                index,
                p_delta,
                g_delta: self.calculate_gain_delta(buffer_len, index),
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

    pub fn update(&mut self, mut frequency: f64, gain: f64, osc_type: OscType, silence_next: bool) {
        if frequency < 20.0 {
            frequency = 0.0;
        }

        let mut gain = if frequency != 0.0 { gain } else { 0.0 };
        if osc_type != OscType::Sine {
            gain /= 3.0
        }
        let loudness = loudness_normalization(frequency);
        gain *= loudness;

        if self.osc_type == OscType::Sine && osc_type == OscType::Noise {
            self.past.gain = self.current.gain / 3.0;
        } else {
            self.past.gain = self.current.gain
        }


        self.osc_type = osc_type;
        self.past.frequency = self.current.frequency;
        self.current.frequency = frequency;
        self.current.gain = gain;

        if self.silent() {
            match self.asr {
                ASR::SR | ASR::ASR | ASR::Silence => {
                    self.asr = ASR::Silence
                }
                _ => { self.asr = ASR::SR }
            }
        } else {
            match self.asr {
                ASR::Silence | ASR::AS | ASR::S => {
                    if silence_next {
                        self.asr = ASR::SR;
                    } else {
                        self.asr = ASR::S;
                    }
                },
                _ => { self.asr = ASR::AS }
            }
        }
//            ASR::Silence => {
//                if silence_next {
//                    if self.silent() {
//                        self.asr = ASR::Silence
//                    } else {
//                        self.asr = ASR::AS;
//                    }
//                } else {
//                    if self.asr {
//
//                    }
//                }
//            }
//        }
//        }

        println!("{:?}, {:?}, {:?}", self.silent(), silence_next, self.asr);
    }

    fn silent(&self) -> bool {
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

    pub fn calculate_gain_delta(
        &mut self,
        fade_length: usize,
        index: usize,
    ) -> f64 {
        //        println!("{:?}, {:?}, {:?}, {:?}", self.current.gain, self.past.gain, silence_next, (next) );
//        if self.silence_to_sound() {
//            if self.counter < self.attack {
//                return index as f64 * (self.current.gain - self.past.gain) / self.attack as f64
//            } else {
//                return self.current.gain
//            }
//        }

        index as f64 * (self.current.gain - self.past.gain) / fade_length as f64
    }
}
