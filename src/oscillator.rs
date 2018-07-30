use ratios::{Pan, R};
use settings::Settings;
use std::f32::consts::PI;
fn tau() -> f32 {
    PI * 2.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    index: usize,
    ratio: R,
    past: VoiceState,
    current: VoiceState,
    phase: f32,
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
    fn init(index: usize, ratio: R) -> Voice {
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
        if (self.current.gain - new_gain).abs() > 0.5 {
            new_gain = new_gain * 0.51;
        }

        self.past = self.current.clone();
        self.current.frequency = new_freq;
        self.current.gain = new_gain;
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

        let gain = ((index as f32 * g_delta) + self.past.gain) * self.ratio.gain;
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

pub fn freq_to_sones(frequency: f32) -> f32 {
    // http://www.ukintpress-conferences.com/conf/08txeu_conf/pdf/day_1/01-06-garcia.pdf
    if frequency < 20.0 {
        0.0
    } else {
        1.0 / 2.0_f32.powf(((20.0 * (frequency).log10()) - 40.0) / 10.0)
    }
}

pub fn loudness_normalization(frequency: f32) -> f32 {
    let mut normalization = freq_to_sones(frequency);
    if normalization.is_nan() || normalization.is_infinite() || normalization > 1.0 {
        normalization = 1.0;
    };
    normalization
}

#[derive(Clone, Debug)]
pub struct StereoWaveform {
    pub l_buffer: Vec<f32>,
    pub r_buffer: Vec<f32>,
}

impl StereoWaveform {
    pub fn new(buffer_size: usize) -> StereoWaveform {
        StereoWaveform {
            l_buffer: vec![0.0; buffer_size],
            r_buffer: vec![0.0; buffer_size],
        }
    }

    pub fn append(&mut self, mut stereo_waveform: StereoWaveform) {
        self.l_buffer.append(&mut stereo_waveform.l_buffer);
        self.r_buffer.append(&mut stereo_waveform.r_buffer);
    }

    pub fn get_buffer(&mut self, index: usize, buffer_size: usize) -> StereoWaveform {
        if (index + 1) * buffer_size < self.l_buffer.len() {
            let l_buffer = &self.l_buffer[index * buffer_size..(index + 1) * buffer_size];
            let r_buffer = &self.r_buffer[index * buffer_size..(index + 1) * buffer_size];
            StereoWaveform {
                l_buffer: l_buffer.to_vec(),
                r_buffer: r_buffer.to_vec() }
            } else {
            StereoWaveform::new(2048)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewOscillator {
    voices: Vec<Voice>,
    portamento_length: usize,
    settings: Settings,
}

impl NewOscillator {
    pub fn init(ratios: Vec<R>, settings: &Settings) -> NewOscillator {
        let voices = ratios
            .iter()
            .enumerate()
            .map(|(index, ratio)| Voice::init(index, ratio.clone()))
            .collect::<Vec<Voice>>();
        NewOscillator {
            voices,
            portamento_length: settings.buffer_size,
            settings: settings.clone(),
        }
    }
    pub fn update_freq_and_gain(&mut self, base_frequency: f32, gain: f32) {
        let new_freq =
            if base_frequency < self.settings.max_freq && base_frequency > self.settings.min_freq {
                base_frequency
            } else {
                0.0
            };

        let new_gain = if gain > self.settings.gain_threshold_min {
            gain
        } else {
            0.0
        };

        for voice in self.voices.iter_mut() {
            voice.update(new_freq, new_gain);
        }
    }

    pub fn update_ratios(&mut self, ratios: &Vec<R>) {
        for (voice, ratio) in self.voices.iter_mut().zip(ratios) {
            voice.ratio = ratio.clone();
        }
    }

    pub fn update_freq_gain_and_ratios(&mut self, base_frequency: f32, gain: f32, ratios: &Vec<R>) {
        self.update_freq_and_gain(base_frequency, gain);
        self.update_ratios(&ratios)
    }

    pub fn generate(&mut self, length: usize) -> StereoWaveform {
        let mut l_buffer: Vec<f32> = vec![0.0; length];
        let mut r_buffer: Vec<f32> = vec![0.0; length];
        let factor: f32 = tau() / self.settings.sample_rate;
        for voice in self.voices.iter_mut() {
            if voice.ratio.pan == Pan::Left {
                voice.generate_waveform(&mut l_buffer, self.portamento_length, factor);
            } else {
                voice.generate_waveform(&mut r_buffer, self.portamento_length, factor);
            }
        }

        StereoWaveform { l_buffer, r_buffer }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use settings::get_test_settings;
    #[test]
    fn oscillator_init_test() {
        let osc = NewOscillator::init(simple_ratios(), &get_test_settings());
        println!("{:?}", osc);
        let expected = vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 50];
        assert_eq!(osc, expected);
    }
}
