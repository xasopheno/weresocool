use oscillator::StereoWaveform;
use ratios::{R, Pan};
use settings::Settings;
use std::cmp::min;
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct NewOscillator {
    voices: Vec<Voice>,
    base_frequency: f32,
    portamento_length: usize,
    settings: Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    index: usize,
    ratio: R,
    past: VoiceState,
    current: VoiceState,
    phase: f32,
    pan: Pan,
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
    fn init(index: usize, ratio: R, pan: Pan) -> Voice {
        Voice {
            index,
            ratio,
            past: VoiceState::init(),
            current: VoiceState::init(),
            phase: 0.0,
            pan
        }
    }
    pub fn generate_waveform(
        &mut self,
        mut buffer: &mut Vec<f32>,
        portamento_length: usize,
        factor: f32,
    ) {
        let p_delta: f32 = self.calculate_portamento_delta(portamento_length);
        let g_delta = self.calculate_gain_delta(buffer.len());

        buffer.iter_mut().enumerate().map(|(index, sample)| {
            *sample += self.generate_sample(index, p_delta, g_delta, portamento_length, factor);
        });
    }

    pub fn update(&mut self, base_frequency: f32, gain: f32) {
        let new_freq = base_frequency * self.ratio.decimal;
        let mut new_gain = if new_freq != 0.0 { gain } else { 0.0 };

        self.past = self.current.clone();
        self.current.frequency = new_freq;
        self.current.gain = new_gain;
    }

    pub fn generate_sample(
        &mut self,
        index: usize,
        p_delta: f32,
        g_delta: f32,
        portamento_length: usize,
        factor: f32,
    ) -> f32 {
        let frequency = if index < portamento_length {
            self.past.frequency * index as f32 * p_delta
        } else {
            self.current.frequency
        };

        let gain = index as f32 * g_delta + self.past.gain;

        let sample =
            (((index as f32 * factor * frequency) + self.phase) % tau()).sin() * self.current.gain;

        self.phase += self.calculate_individual_phase(frequency, 1.0, factor, self.phase);
        sample
    }

    fn calculate_portamento_delta(&self, portamento_length: usize) -> f32 {
        (self.current.frequency - self.past.frequency) / (portamento_length as f32 - 1.0)
    }

    fn calculate_gain_delta(&self, buffer_size: usize) -> f32 {
        (self.current.gain - self.past.gain) / (buffer_size as f32 - 1.0)
    }

    fn calculate_individual_phase(
        &self,
        frequency: f32,
        buffer_size: f32,
        factor: f32,
        phase: f32,
    ) -> f32 {
        ((buffer_size as f32 * factor * frequency) + phase) % tau()
    }
}

fn tau() -> f32 {
    PI * 2.0
}

impl NewOscillator {
    pub fn init(ratios: Vec<R>, settings: &Settings) -> NewOscillator {
        Voice::init(index: usize, ratio: R, pan: Pan);

    }
    pub fn update(&mut self, frequency: f32, gain: f32) {
        // TODO: implement frequency threshold
        let new_freq = if frequency < 2_500.0 && frequency > 0.0 {
            frequency
        } else {
            0.0
        };

        // TODO: implement gain threshold
        let new_gain = if gain < 0.0 { gain } else { 0.0 };

        self.voices
            .iter_mut()
            .map(|voice| voice.update(frequency, gain));
    }

    pub fn generate(&mut self) -> (Vec<f32>, Vec<f32>) {
        let mut l_buffer: Vec<f32> = vec![0.0; self.settings.buffer_size];
        let mut r_buffer: Vec<f32> = vec![0.0; self.settings.buffer_size];
        let portamento_length = self.portamento_length;
        let factor: f32 = tau() / self.settings.sample_rate;
        self.voices.iter_mut().map(|voice| {
            if voice.pan == Pan::Left {
                voice.generate_waveform(&mut l_buffer, portamento_length, factor);
            } else {
                voice.generate_waveform(&mut r_buffer, portamento_length, factor);
            }
        });

        (l_buffer, r_buffer)
    }
}
