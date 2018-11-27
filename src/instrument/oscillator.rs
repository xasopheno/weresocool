use event::Sound;
use instrument::{stereo_waveform::StereoWaveform, voice::Voice};
use settings::Settings;
use std::f32::consts::PI;
fn tau() -> f32 {
    PI * 2.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Oscillator {
    pub voices: Vec<(Voice, Voice)>,
    pub portamento_length: usize,
    pub settings: Settings,
    sample_phase: f32,
}

impl Oscillator {
    pub fn init(settings: &Settings) -> Oscillator {
        Oscillator {
            voices: vec![(Voice::init(0), Voice::init(1))],
            portamento_length: settings.buffer_size,
            settings: settings.clone(),
            sample_phase: 0.0,
        }
    }

    pub fn update(&mut self, mut sounds: Vec<Sound>) {
        let len_voices = self.voices.len();
        let len_sounds = sounds.len();
        let difference = (len_voices as isize - len_sounds as isize).abs();

        if len_sounds > len_voices {
            for i in 0..difference {
                self.voices.push((
                    Voice::init(len_voices + i as usize + 1),
                    Voice::init(len_voices + i as usize + 2),
                ));
            }
        }
        if len_sounds < len_voices {
            for _ in 0..difference {
                sounds.push(Sound::init());
            }
        }

        for (sound, lr_voice) in sounds.iter().zip(self.voices.iter_mut()) {
            let l_gain = sound.gain * ((-1.0 + sound.pan) / -2.0);
            let r_gain = sound.gain * ((1.0 + sound.pan) / 2.0);
            let (ref mut l_voice, ref mut r_voice) = lr_voice;
            l_voice.update(sound.frequency, l_gain);
            r_voice.update(sound.frequency, r_gain);
        }
    }

    pub fn generate(&mut self, n_samples_to_generate: f32) -> StereoWaveform {
        let total_len = self.sample_phase + n_samples_to_generate;
        let length = total_len.floor() as usize;
        self.sample_phase = total_len.fract();
        let mut l_buffer: Vec<f32> = vec![0.0; length];
        let mut r_buffer: Vec<f32> = vec![0.0; length];
        let factor: f32 = tau() / self.settings.sample_rate;
        for lr_voices in self.voices.iter_mut() {
            let (ref mut l_voice, ref mut r_voice) = *lr_voices;
            l_voice.generate_waveform(&mut l_buffer, self.portamento_length, factor);
            r_voice.generate_waveform(&mut r_buffer, self.portamento_length, factor);
        }

        StereoWaveform { l_buffer, r_buffer }
    }
}
