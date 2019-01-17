extern crate num_rational;
use generation::parsed_to_render::r_to_f64;
use instrument::{stereo_waveform::StereoWaveform, voice::Voice};
use operations::PointOp;
use settings::Settings;
use std::f64::consts::PI;
fn tau() -> f64 {
    PI * 2.0
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub enum OscType {
    Sine,
    Noise,
    Square,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Oscillator {
    pub voices: (Voice, Voice),
    pub portamento_length: usize,
    pub settings: Settings,
    pub sample_phase: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Origin {
    pub f: f64,
    pub p: f64,
    pub g: f64,
    pub l: f64,
}

impl Oscillator {
    pub fn init(settings: &Settings) -> Oscillator {
        Oscillator {
            voices: (Voice::init(0), Voice::init(1)),
            portamento_length: settings.buffer_size,
            settings: settings.clone(),
            sample_phase: 0.0,
        }
    }

    pub fn update(&mut self, basis: Origin, point_op: &PointOp) {
        let pm = r_to_f64(point_op.pm);

        let l_gain = if *point_op.g.numer() == 0 {
            0.0
        } else {
            r_to_f64(point_op.g) * (((-1.0 + r_to_f64(point_op.pa) * pm) + basis.p) / -2.0) * basis.g
        };

        let r_gain = if *point_op.g.numer() == 0 {
            0.0
        } else {
            r_to_f64(point_op.g) * (((1.0 + r_to_f64(point_op.pa) * pm) + basis.p) / 2.0) * basis.g
        };
        let (ref mut l_voice, ref mut r_voice) = self.voices;

        l_voice.update(
            (basis.f * r_to_f64(point_op.fm)) + r_to_f64(point_op.fa),
            l_gain,
            point_op.osc_type,
        );
        r_voice.update(
            (basis.f * r_to_f64(point_op.fm)) + r_to_f64(point_op.fa),
            r_gain,
            point_op.osc_type,
        );
    }

    pub fn generate(&mut self, n_samples_to_generate: f64) -> StereoWaveform {
        let total_len = self.sample_phase + n_samples_to_generate;
        let length = total_len.floor() as usize;
        self.sample_phase = total_len.fract();
        let mut l_buffer: Vec<f64> = vec![0.0; length];
        let mut r_buffer: Vec<f64> = vec![0.0; length];
        let factor: f64 = tau() / self.settings.sample_rate;

        let (ref mut l_voice, ref mut r_voice) = self.voices;
        l_voice.generate_waveform(&mut l_buffer, self.portamento_length, factor);
        r_voice.generate_waveform(&mut r_buffer, self.portamento_length, factor);

        StereoWaveform { l_buffer, r_buffer }
    }
}
