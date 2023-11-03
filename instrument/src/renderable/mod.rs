pub mod render_voice;

use crate::{Basis, Oscillator, StereoWaveform};
use num_rational::Rational64;
use num_traits::CheckedMul;
use rand::{thread_rng, Rng};
#[cfg(feature = "app")]
use rayon::prelude::*;
pub use render_voice::{renderables_to_render_voices, RenderVoice};
use scop::Defs;
use serde::{Deserialize, Serialize};
use weresocool_ast::{NormalForm, Normalize, OscType, PointOp, Term, ASR};
use weresocool_error::Error;
use weresocool_filter::BiquadFilterDef;
pub(crate) use weresocool_shared::{lossy_rational_mul, r_to_f64, Settings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderOp {
    pub f: f64,
    pub p: f64,
    pub l: f64,
    pub g: (f64, f64),
    /// Time
    pub t: f64,
    pub attack: f64,
    pub decay: f64,
    pub asr: ASR,
    pub samples: usize,
    pub index: usize,
    pub total_samples: usize,
    pub voice: usize,
    pub event: usize,
    pub portamento: usize,
    pub reverb: Option<f64>,
    pub osc_type: OscType,
    pub next_l_silent: bool,
    pub next_r_silent: bool,
    pub names: Vec<String>,
    pub filters: Vec<BiquadFilterDef>,
    pub next_out: bool,
}

impl RenderOp {
    pub const fn init_fglp(f: f64, g: (f64, f64), l: f64, p: f64, settings: &Settings) -> Self {
        Self {
            f,
            p,
            g,
            l,
            t: 0.0,
            reverb: None,
            attack: settings.sample_rate,
            decay: settings.sample_rate,
            asr: ASR::Long,
            samples: settings.sample_rate as usize,
            total_samples: settings.sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: false,
            next_r_silent: false,
            next_out: false,
            names: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub const fn init_fglp_samples(
        f: f64,
        g: (f64, f64),
        l: f64,
        p: f64,
        total_samples: usize,
        settings: &Settings,
    ) -> Self {
        Self {
            f,
            p,
            g,
            l,
            t: 0.0,
            reverb: None,
            attack: settings.sample_rate,
            decay: settings.sample_rate,
            asr: ASR::Long,
            samples: total_samples,
            total_samples,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: false,
            next_r_silent: false,
            next_out: false,
            names: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn init_silent_with_length(l: f64) -> Self {
        Self {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            reverb: None,
            attack: Settings::global().sample_rate,
            decay: Settings::global().sample_rate,
            asr: ASR::Long,
            samples: Settings::global().sample_rate as usize,
            total_samples: Settings::global().sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: true,
            next_r_silent: true,
            next_out: false,
            names: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub const fn init_silent_with_length_osc_type_reverb_and_filters(
        l: f64,
        osc_type: OscType,
        reverb: Option<f64>,
        filters: Vec<BiquadFilterDef>,
        sample_rate: f64,
    ) -> Self {
        Self {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            reverb,
            attack: sample_rate,
            decay: sample_rate,
            asr: ASR::Long,
            samples: sample_rate as usize,
            total_samples: sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type,
            next_l_silent: true,
            next_r_silent: true,
            next_out: false,
            names: vec![],
            filters,
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
            freq: thread_rng().gen_range(0.95, 1.05),
            gain: 1.0,
        }
    }
}

pub trait Renderable<T> {
    fn render(&self, oscillator: &mut Oscillator, _offset: Option<&Offset>) -> StereoWaveform;
}

impl Renderable<RenderOp> for RenderOp {
    fn render(&self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
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
    fn render(&self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);

        for op in self.iter() {
            if op.samples > 0 {
                let stereo_waveform = op.render(oscillator, offset);
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
    let settings = Settings::global();
    let mut next_l_gain = 0.0;
    let mut next_r_gain = 0.0;
    let mut next_out = false;
    let _next_ = false;
    let next_silent;

    match next {
        Some(op) => {
            let (l, r) = point_op_to_gains(&op, basis, 1.0, 1.0);
            next_l_gain = l;
            next_r_gain = r;
            next_silent = op.is_silent();
            next_out = op.is_out;
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
        reverb: if point_op.reverb.is_some() {
            Some(r_to_f64(point_op.reverb.unwrap()))
        } else {
            None
        },
        index: 0,
        samples: (l * settings.sample_rate).round() as usize,
        total_samples: (l * settings.sample_rate).round() as usize,
        attack: r_to_f64(point_op.attack * basis.a) * settings.sample_rate,
        decay: r_to_f64(point_op.decay * basis.d) * settings.sample_rate,
        osc_type: point_op.osc_type.clone(),
        asr: point_op.asr,
        portamento: (r_to_f64(point_op.portamento) * 1024_f64) as usize,
        voice,
        event,
        next_l_silent,
        next_r_silent,
        names: point_op.names.to_vec(),
        filters: point_op
            .filters
            .iter()
            .map(|f| BiquadFilterDef {
                hash: f.hash.clone(),
                filter_type: f.filter_type,
                cutoff_frequency: f.cutoff_frequency * basis.f,
                q_factor: f.q_factor,
            })
            .collect(),
        next_out,
    };

    *time += point_op.l * basis.l;

    render_op
}

pub fn point_op_to_gains(
    point_op: &PointOp,
    basis: &Basis,
    angle: f64,
    _frequency: f64,
) -> (f64, f64) {
    if *point_op.g.numer() == 0 {
        return (0.0, 0.0);
    }

    // let f = m_a_and_basis_to_f64(basis.f, point_op.fm, point_op.fa);

    let pm = r_to_f64(point_op.pm);
    let pa = r_to_f64(point_op.pa);
    let g = r_to_f64(point_op.g);
    let base_p = r_to_f64(basis.p);
    let base_g = r_to_f64(basis.g);

    // let ild = calculate_ild(angle, f);

    // let l_gain = g * (((pa.mul_add(pm, 1.0 + ild)) + base_p) / 2.0) * base_g;
    // let r_gain = g * (((pa.mul_add(pm, -1.0 - ild)) + base_p) / -2.0) * base_g;
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

fn calculate_ild(angle: f64, frequency: f64) -> f64 {
    const MAX_ILD: f64 = 1.0;
    const FREQUENCY_FACTOR: f64 = 0.001;

    let angle_factor = (angle / 90.0).cos();
    let frequency_factor = frequency * FREQUENCY_FACTOR;

    MAX_ILD * angle_factor * frequency_factor
}

pub fn m_a_and_basis_to_f64(basis: Rational64, m: Rational64, a: Rational64) -> f64 {
    r_to_f64(
        basis
            .checked_mul(&m)
            .unwrap_or_else(|| lossy_rational_mul(basis, m)),
    ) + r_to_f64(a)
}

pub fn float_to_angle(input: f64) -> f64 {
    // Map the input in the range -1 to 1 into the range -90 to 90.
    input * 90.0
}

pub fn calculate_fgpl(basis: &Basis, point_op: &PointOp) -> (f64, (f64, f64), f64, f64) {
    let settings = Settings::global();
    let p = m_a_and_basis_to_f64(basis.p, point_op.pm, point_op.pa);

    let (mut f, mut g) = if point_op.is_silent() {
        (0.0, (0.0, 0.0))
    } else {
        let f = m_a_and_basis_to_f64(basis.f, point_op.fm, point_op.fa);
        let g = point_op_to_gains(point_op, basis, f, float_to_angle(p));
        (f, g)
    };
    let l = r_to_f64(point_op.l * basis.l);
    if f < settings.min_freq {
        f = 0.0;
        g = (0.0, 0.0);
    };

    (f, g, p, l)
}

pub fn nf_to_vec_renderable(
    composition: &NormalForm,
    defs: &mut Defs<Term>,
    basis: &Basis,
) -> Result<Vec<Vec<RenderOp>>, Error> {
    let mut normal_form = NormalForm::init();
    composition.apply_to_normal_form(&mut normal_form, defs)?;

    #[cfg(feature = "app")]
    let iter = normal_form.operations.par_iter();
    #[cfg(feature = "wasm")]
    let iter = normal_form.operations.iter();

    let settings = Settings::global();

    let result: Vec<Vec<RenderOp>> = iter
        .enumerate()
        .map(|(voice, vec_point_op)| {
            create_render_ops(
                voice,
                vec_point_op,
                basis,
                settings.sample_rate,
                settings.pad_end,
            )
        })
        .collect();

    Ok(result)
}

pub fn nf_to_vec_renderable_loopable(
    composition: &NormalForm,
    defs: &mut Defs<Term>,
    basis: &Basis,
    looped: bool,
) -> Result<Vec<Vec<RenderOp>>, Error> {
    let mut normal_form = NormalForm::init();
    composition.apply_to_normal_form(&mut normal_form, defs)?;

    #[cfg(feature = "app")]
    let iter = normal_form.operations.par_iter();
    #[cfg(feature = "wasm")]
    let iter = normal_form.operations.iter();

    let settings = Settings::global();

    let result: Vec<Vec<RenderOp>> = iter
        .enumerate()
        .map(|(voice, vec_point_op)| {
            create_render_ops(voice, vec_point_op, basis, settings.sample_rate, !looped)
        })
        .collect();

    Ok(result)
}

fn create_render_ops(
    voice: usize,
    vec_point_op: &[PointOp],
    basis: &Basis,
    sample_rate: f64,
    pad_end: bool,
) -> Vec<RenderOp> {
    let mut time = Rational64::new(0, 1);
    let mut result: Vec<RenderOp> = vec![];

    for (event, p_op) in vec_point_op.iter().enumerate() {
        let next_e = if event == vec_point_op.len() - 1 {
            0
        } else {
            event + 1
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

    if pad_end {
        let filters = if let Some(last_op) = vec_point_op.last() {
            last_op.filters.to_vec()
        } else {
            vec![]
        };
        result.push(
            RenderOp::init_silent_with_length_osc_type_reverb_and_filters(
                1.0,
                OscType::None,
                None,
                filters,
                sample_rate,
            ),
        );
    }

    result
}
