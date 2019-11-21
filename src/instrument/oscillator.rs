use crate::{
    instrument::{stereo_waveform::StereoWaveform, voice::Voice},
    renderable::RenderOp,
    settings::Settings,
};
use num_rational::Rational64;
use socool_parser::Init;

#[derive(Clone, Debug, PartialEq)]
pub struct Oscillator {
    pub voices: (Voice, Voice),
    pub settings: Settings,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Basis {
    pub f: Rational64,
    pub p: Rational64,
    pub g: Rational64,
    pub l: Rational64,
    pub a: Rational64,
    pub d: Rational64,
}

impl From<Init> for Basis {
    fn from(init: Init) -> Basis {
        Basis {
            f: init.f,
            g: init.g,
            l: init.l,
            p: init.p,
            a: Rational64::new(1, 1),
            d: Rational64::new(1, 1),
        }
    }
}

impl Oscillator {
    pub fn init(settings: &Settings) -> Oscillator {
        Oscillator {
            voices: (Voice::init(0), Voice::init(1)),
            settings: settings.clone(),
        }
    }

    pub fn update(&mut self, op: &RenderOp) {
        let (ref mut l_voice, ref mut r_voice) = self.voices;
        l_voice.update(op);
        r_voice.update(op);
    }

    pub fn generate(&mut self, op: &RenderOp) -> StereoWaveform {
        let mut l_buffer: Vec<f64> = vec![0.0; op.samples];
        let mut r_buffer: Vec<f64> = vec![0.0; op.samples];

        let (ref mut l_voice, ref mut r_voice) = self.voices;

        l_voice.generate_waveform(
            &mut l_buffer,
            op.portamento,
            op.index,
            op.total_samples,
            op.next_l_silent,
        );
        r_voice.generate_waveform(
            &mut r_buffer,
            op.portamento,
            op.index,
            op.total_samples,
            op.next_r_silent,
        );

        StereoWaveform { l_buffer, r_buffer }
    }
}
