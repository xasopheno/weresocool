pub mod render_voice;

use crate::{Basis, Oscillator, StereoWaveform};
use num_rational::Rational64;
use num_traits::CheckedMul;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
pub use render_voice::{renderables_to_render_voices, RenderVoice};
use weresocool_ast::{Defs, NormalForm, Normalize, OscType, PointOp, ASR};
use weresocool_error::Error;
use weresocool_shared::{default_settings, lossy_rational_mul, Settings};

const SETTINGS: Settings = default_settings();

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderOp {
    pub f: f64,
    pub p: f64,
    pub l: f64,
    pub g: (f64, f64),
    pub t: f64,
    pub reverb: f64,
    pub attack: f64,
    pub decay: f64,
    pub asr: ASR,
    pub samples: usize,
    pub index: usize,
    pub total_samples: usize,
    pub voice: usize,
    pub event: usize,
    pub portamento: usize,
    pub osc_type: OscType,
    pub next_l_silent: bool,
    pub next_r_silent: bool,
}

impl RenderOp {
    pub const fn init_fglp(f: f64, g: (f64, f64), l: f64, p: f64) -> Self {
        Self {
            f,
            p,
            g,
            l,
            t: 0.0,
            reverb: 0.0,
            attack: SETTINGS.sample_rate,
            decay: SETTINGS.sample_rate,
            asr: ASR::Long,
            samples: SETTINGS.sample_rate as usize,
            total_samples: SETTINGS.sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: false,
            next_r_silent: false,
        }
    }
    pub const fn init_silent_with_length(l: f64) -> Self {
        Self {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            reverb: 0.0,
            attack: SETTINGS.sample_rate,
            decay: SETTINGS.sample_rate,
            asr: ASR::Long,
            samples: SETTINGS.sample_rate as usize,
            total_samples: SETTINGS.sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: true,
            next_r_silent: true,
        }
    }

    pub const fn init_silent_with_length_and_osctype(
        l: f64,
        osc_type: OscType,
        reverb: f64,
    ) -> Self {
        Self {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            reverb,
            attack: SETTINGS.sample_rate,
            decay: SETTINGS.sample_rate,
            asr: ASR::Long,
            samples: SETTINGS.sample_rate as usize,
            total_samples: SETTINGS.sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type,
            next_l_silent: true,
            next_r_silent: true,
        }
    }
}

#[derive(Debug)]
pub struct Offset {
    pub freq: f64,
    pub gain: f64,
}
impl Offset {
    pub const fn identity() -> Self {
        Self {
            freq: 1.0,
            gain: 1.0,
        }
    }
    pub fn random() -> Self {
        Self {
            freq: thread_rng().gen_range(0.95..1.05),
            gain: 1.0,
        }
    }
}

pub trait Renderable<T> {
    fn render(&mut self, oscillator: &mut Oscillator, _offset: Option<&Offset>) -> StereoWaveform;
}

impl Renderable<RenderOp> for RenderOp {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let o = match offset {
            Some(o) => Offset {
                freq: o.freq * 2.0,
                gain: o.gain,
            },
            None => Offset::identity(),
        };

        oscillator.update(self, &o);
        oscillator.generate(self, &o)
    }
}
impl Renderable<Vec<RenderOp>> for Vec<RenderOp> {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);

        for op in self.iter() {
            if op.samples > 0 {
                let stereo_waveform = op.clone().render(oscillator, offset);
                result.append(stereo_waveform);
            }
        }

        result
    }
}

fn pointop_to_renderop(
    point_op: &PointOp,
    time: &mut Rational64,
    voice: usize,
    event: usize,
    basis: &Basis,
    next: Option<PointOp>,
) -> RenderOp {
    let mut next_l_gain = 0.0;
    let mut next_r_gain = 0.0;
    let next_silent;

    match next {
        Some(op) => {
            let (l, r) = point_op_to_gains(&op, basis);
            next_l_gain = l;
            next_r_gain = r;
            next_silent = op.is_silent();
        }

        None => next_silent = true,
    }

    let next_l_silent = next_silent || next_l_gain == 0.0;
    let next_r_silent = next_silent || next_r_gain == 0.0;

    let (f, g, p, l) = calculate_fgpl(basis, point_op);

    let render_op = RenderOp {
        f,
        g,
        p,
        l,
        t: r_to_f64(*time),
        reverb: r_to_f64(point_op.reverb),
        index: 0,
        samples: (l * SETTINGS.sample_rate).round() as usize,
        total_samples: (l * SETTINGS.sample_rate).round() as usize,
        attack: r_to_f64(point_op.attack * basis.a) * SETTINGS.sample_rate,
        decay: r_to_f64(point_op.decay * basis.d) * SETTINGS.sample_rate,
        osc_type: point_op.osc_type,
        asr: point_op.asr,
        portamento: (r_to_f64(point_op.portamento) * 1024_f64) as usize,
        voice,
        event,
        next_l_silent,
        next_r_silent,
    };

    *time += point_op.l * basis.l;

    render_op
}

pub fn point_op_to_gains(point_op: &PointOp, basis: &Basis) -> (f64, f64) {
    let pm = r_to_f64(point_op.pm);
    let pa = r_to_f64(point_op.pa);
    let g = r_to_f64(point_op.g);

    let l_gain = if *point_op.g.numer() == 0 {
        0.0
    } else {
        g * (((pa.mul_add(pm, 1.0)) + r_to_f64(basis.p)) / 2.0) * r_to_f64(basis.g)
    };

    let r_gain = if *point_op.g.numer() == 0 {
        0.0
    } else {
        g * (((pa.mul_add(pm, -1.0)) + r_to_f64(basis.p)) / -2.0) * r_to_f64(basis.g)
    };

    (l_gain, r_gain)
}

pub fn m_a_and_basis_to_f64(basis: Rational64, m: Rational64, a: Rational64) -> f64 {
    r_to_f64(
        basis
            .checked_mul(&m)
            .unwrap_or_else(|| lossy_rational_mul(basis, m)),
    ) + r_to_f64(a)
}

pub fn calculate_fgpl(basis: &Basis, point_op: &PointOp) -> (f64, (f64, f64), f64, f64) {
    let (mut f, mut g) = if point_op.is_silent() {
        (0.0, (0.0, 0.0))
    } else {
        let g = point_op_to_gains(point_op, basis);
        (m_a_and_basis_to_f64(basis.f, point_op.fm, point_op.fa), g)
    };
    let p = m_a_and_basis_to_f64(basis.p, point_op.pm, point_op.pa);
    let l = r_to_f64(point_op.l * basis.l);
    if f < SETTINGS.min_freq {
        f = 0.0;
        g = (0.0, 0.0);
    };

    (f, g, p, l)
}

pub fn nf_to_vec_renderable(
    composition: &NormalForm,
    defs: &Defs,
    basis: &Basis,
) -> Result<Vec<Vec<RenderOp>>, Error> {
    let mut normal_form = NormalForm::init();
    composition.apply_to_normal_form(&mut normal_form, defs)?;

    let result: Vec<Vec<RenderOp>> = normal_form
        .operations
        .par_iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
            let last_op = vec_point_op
                .last()
                .cloned()
                .unwrap_or_else(PointOp::init_silent);
            let last_osc = last_op.osc_type;
            let last_reverb = last_op.reverb;

            let mut time = Rational64::new(0, 1);
            let mut result: Vec<RenderOp> = vec![];
            for (event, p_op) in vec_point_op.iter().enumerate() {
                let mut next_e = event;
                if event == vec_point_op.len() {
                    next_e = 0;
                };

                let op = pointop_to_renderop(
                    p_op,
                    &mut time,
                    voice,
                    event,
                    basis,
                    Some(vec_point_op[next_e].clone()),
                );
                result.push(op);
            }
            if default_settings().pad_end {
                result.push(RenderOp::init_silent_with_length_and_osctype(
                    1.0,
                    OscType::None,
                    weresocool_shared::helpers::r_to_f64(last_reverb),
                ));
            }
            result
        })
        .collect();

    Ok(result)
}
