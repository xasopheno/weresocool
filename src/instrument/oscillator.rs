use crate::{
    generation::parsed_to_render::r_to_f64,
    instrument::{
        stereo_waveform::StereoWaveform,
        voice::{Voice, VoiceUpdate},
    },
    renderable::{Offset, RenderOp},
    settings::Settings,
};
use num_rational::Rational64;
use socool_ast::PointOp;
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

pub fn point_op_to_gains(point_op: &PointOp, basis: &Basis) -> (f64, f64) {
    let pm = r_to_f64(point_op.pm);
    let pa = r_to_f64(point_op.pa);
    let g = r_to_f64(point_op.g);

    let l_gain = if *point_op.g.numer() == 0 {
        0.0
    } else {
        g * (((1.0 + pa * pm) + r_to_f64(basis.p)) / 2.0) * r_to_f64(basis.g)
    };

    let r_gain = if *point_op.g.numer() == 0 {
        0.0
    } else {
        g * (((-1.0 + pa * pm) + r_to_f64(basis.p)) / -2.0) * r_to_f64(basis.g)
    };

    (l_gain, r_gain)
}

impl Oscillator {
    pub fn init(settings: &Settings) -> Oscillator {
        Oscillator {
            voices: (Voice::init(0), Voice::init(1)),
            settings: settings.clone(),
        }
    }

    pub fn update(&mut self, op: &RenderOp, start: bool) {
        let (ref mut l_voice, ref mut r_voice) = self.voices;

        l_voice.update(
            VoiceUpdate {
                frequency: op.f,
                gain: op.g.0,
                osc_type: op.osc_type,
                silence_next: op.next_l_silent,
                attack: op.attack,
                decay: op.decay,
                decay_type: op.decay_length,
            },
            start,
        );
        r_voice.update(
            VoiceUpdate {
                frequency: op.f,
                gain: op.g.1,
                osc_type: op.osc_type,
                silence_next: op.next_r_silent,
                attack: op.attack,
                decay: op.decay,
                decay_type: op.decay_length,
            },
            start,
        );
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
