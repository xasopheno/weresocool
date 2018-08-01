use oscillator::loudness::loudness_normalization;
use ratios::R;
use std::f32::consts::PI;
fn tau() -> f32 {
    PI * 2.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub ratio: R,
    pub past: VoiceState,
    pub current: VoiceState,
    pub phase: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceState {
    frequency: f32,
    gain: f32,
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
    pub fn init(index: usize, ratio: R) -> Voice {
        Voice {
            index,
            ratio,
            past: VoiceState::init(),
            current: VoiceState::init(),
            phase: 0.0,
        }
    }
    pub fn generate_waveform(
        &mut self,
        buffer: &mut Vec<f32>,
        portamento_length: usize,
        factor: f32,
    ) {
        let p_delta = self.calculate_portamento_delta(portamento_length);
        let g_delta = self.calculate_gain_delta(buffer.len());
        for (index, sample) in buffer.iter_mut().enumerate() {
            let new_sample =
                self.generate_sample(index, p_delta, g_delta, portamento_length, factor);
            *sample += new_sample
        }
    }

    pub fn update(&mut self, base_frequency: f32, gain: f32) {
        let mut new_freq = base_frequency * self.ratio.decimal + self.ratio.offset;
        if new_freq < 20.0 {
            new_freq = 0.0;
        }

        let mut new_gain = if new_freq != 0.0 { gain } else { 0.0 };
        let loudness = loudness_normalization(new_freq);
        new_gain *= loudness;
        //        if (self.current.gain - new_gain).abs() > 0.5 {
        //            new_gain = new_gain * 0.51;
        //        }

        self.past.frequency = self.current.frequency;
        self.past.gain = self.current.gain;
        self.current.frequency = new_freq;
        self.current.gain = new_gain * self.ratio.gain;
    }

    fn silence_to_sound(&self) -> bool {
        self.past.frequency == 0.0 && self.current.frequency != 0.0
    }

    fn sound_to_silence(&self) -> bool {
        self.past.frequency != 0.0 && self.current.frequency == 0.0
    }

    pub fn generate_sample(
        &mut self,
        index: usize,
        p_delta: f32,
        g_delta: f32,
        portamento_length: usize,
        factor: f32,
    ) -> f32 {
        let frequency = if self.sound_to_silence() {
            self.past.frequency
        } else if index < portamento_length && !self.silence_to_sound() && !self.sound_to_silence()
        {
            self.past.frequency + (index as f32 * p_delta)
        } else {
            self.current.frequency
        };

        let gain = (index as f32 * g_delta) + self.past.gain;
        let current_phase = ((factor * frequency) + self.phase) % tau();
        self.phase = current_phase;

        current_phase.sin() * gain
    }

    fn calculate_portamento_delta(&self, portamento_length: usize) -> f32 {
        (self.current.frequency - self.past.frequency) / (portamento_length as f32)
    }

    fn calculate_gain_delta(&self, buffer_size: usize) -> f32 {
        (self.current.gain - self.past.gain) / (buffer_size as f32)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ratios::{Pan, R};

    #[test]
    fn test_voice_init() {
        let index = 1;
        let ratio = R::atio(3, 2, 0.0, 0.6, Pan::Left);
        let voice = Voice::init(index, ratio.clone());

        let result = Voice {
            ratio,
            index,
            past: VoiceState {
                frequency: 0.0,
                gain: 0.0,
            },
            current: VoiceState {
                frequency: 0.0,
                gain: 0.0,
            },
            phase: 0.0,
        };

        assert_eq!(voice, result);
    }
}
