use instrument::voice::Voice;
use ratios::{Pan, R};
use settings::Settings;
use std::f32::consts::PI;
fn tau() -> f32 {
    PI * 2.0
}

#[derive(Clone, Debug, PartialEq)]
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
                r_buffer: r_buffer.to_vec(),
            }
        } else {
            StereoWaveform::new(2048)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Oscillator {
    pub voices: Vec<Voice>,
    pub portamento_length: usize,
    pub settings: Settings,
}

impl Oscillator {
    pub fn init(ratios: Vec<R>, settings: &Settings) -> Oscillator {
        let voices = ratios
            .iter()
            .enumerate()
            .map(|(index, ratio)| Voice::init(index, ratio.clone()))
            .collect::<Vec<Voice>>();
        Oscillator {
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


