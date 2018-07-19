use settings::{Settings};
use oscillator::StereoWaveform;
use ratios::{R};

#[derive(Clone, Debug, PartialEq)]
pub enum Pan {
    Left,
    Right
}

#[derive(Clone, Debug, PartialEq)]
pub struct Oscillator {
    voices: Vec<Voice>,
    base_frequency: f32,
    buffer_size: usize,
    portamento_length: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    index: usize,
    ratio: R,
    past: VoiceState,
    current: VoiceState,
    phase: f32,
    pan: Pan
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceState {
    frequency: f32,
    gain: f32,
}

impl Voice {
    pub fn generate_waveform(&mut self, mut buffer: &mut Vec<f32>, portamento_length: f32) {
        let p_delta: f32 = self.calculate_portamento_delta(portamento_length);
        let g_delta = self.calculate_gain_delta(buffer.len());

        buffer
            .iter_mut()
            .enumerate()
            .map(|(index, sample)| {
                *sample += self.generate_sample(index, p_delta, g_delta);
            });
    }

    pub fn update(&mut self, base_frequency: f32, gain: f32) {
        
    }

    pub fn generate_sample(&self, index: usize, p_delta: f32, g_delta: f32) -> f32 {
        0.0
    }


    fn calculate_portamento_delta(&self, portamento_length: f32) -> f32 {
        0.0
    }

    fn calculate_gain_delta(&self, buffer_size: usize) -> f32 {
        (self.current.gain - self.past.gain) / (buffer_size as f32 - 1.0)
    }
}

impl Oscillator {
    pub fn update(&mut self, frequency: f32, gain: f32) {
        self.voices
            .iter_mut()
            .map(|voice| {
                voice.update(frequency, gain)
            });
    }

    pub fn generate(&mut self) -> (Vec<f32>, Vec<f32>) {
        let mut l_buffer: Vec<f32> = vec![0.0; self.buffer_size];
        let mut r_buffer: Vec<f32> = vec![0.0; self.buffer_size];
        let portamento_length = self.portamento_length;
        self.voices
            .iter_mut()
            .map(|voice| {
                if voice.pan == Pan::Left {
                    voice.generate_waveform(&mut l_buffer, portamento_length);
                } else {
                    voice.generate_waveform(&mut r_buffer, portamento_length);
                }
            });

        (l_buffer, r_buffer)
    }
}
