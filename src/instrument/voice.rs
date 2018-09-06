use instrument::loudness::loudness_normalization;
use std::f32::consts::PI;
fn tau() -> f32 {
    PI * 2.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub phase: f32,
    //    pub randomness: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceState {
    pub frequency: f32,
    pub gain: f32,
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

    pub fn update(&mut self, mut frequency: f32, gain: f32) {
        if frequency < 20.0 {
            frequency = 0.0;
        }

        let mut gain = if frequency != 0.0 { gain } else { 0.0 };
        let loudness = loudness_normalization(frequency);
        gain *= loudness;

        self.past.frequency = self.current.frequency;
        self.past.gain = self.current.gain;
        self.current.frequency = frequency;
        self.current.gain = gain;
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.frequency == 0.0 && self.current.frequency != 0.0
    }

    pub fn sound_to_silence(&self) -> bool {
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

    pub fn calculate_portamento_delta(&self, portamento_length: usize) -> f32 {
        (self.current.frequency - self.past.frequency) / (portamento_length as f32)
    }

    pub fn calculate_gain_delta(&self, fade_length: usize) -> f32 {
        (self.current.gain - self.past.gain) / (fade_length as f32)
    }
}
