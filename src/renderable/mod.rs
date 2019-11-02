use crate::generation::parsed_to_render::r_to_f64;
use crate::instrument::oscillator::{point_op_to_gains, Basis};
use crate::instrument::{Oscillator, StereoWaveform};
use num_rational::Rational64;
use socool_ast::{NormalForm, Normalize, OpOrNfTable, OscType, PointOp};
pub mod render_voice;
mod test;
pub use render_voice::{renderables_to_render_voices, RenderVoice};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderOp {
    pub f: f64,
    pub p: f64,
    pub l: f64,
    pub g: (f64, f64),
    pub t: f64,
    pub attack: f64,
    pub decay: f64,
    pub decay_length: usize,
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
    pub fn init_fglp(f: f64, g: (f64, f64), l: f64, p: f64) -> RenderOp {
        RenderOp {
            f,
            p,
            g,
            l,
            t: 0.0,
            attack: 44_100.0,
            decay: 44_100.0,
            decay_length: 2,
            samples: 44_100,
            total_samples: 44_100,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::Sine,
            next_l_silent: false,
            next_r_silent: false,
        }
    }
    pub fn init_silent_with_length(l: f64) -> RenderOp {
        RenderOp {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            attack: 44_100.0,
            decay: 44_100.0,
            decay_length: 2,
            samples: 44_100,
            total_samples: 44_100,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::Sine,
            next_l_silent: true,
            next_r_silent: true,
        }
    }
    pub fn apply_offset(&mut self, _offset: &Offset) {
        //self.f = offset.freq * 4.0;
        //self.g = (self.g.0 * offset.gain, self.g.1 * offset.gain);
        self.total_samples = self.samples;

        self.index = 0;
        self.next_r_silent = true;
        self.next_l_silent = true;
    }
}

#[derive(Debug)]
pub struct Offset {
    pub freq: f64,
    pub gain: f64,
}

pub trait Renderable<T> {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform;
}

impl Renderable<RenderOp> for RenderOp {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let mut has_offset = false;
        //match offset {
        //Some(o) => {
        //self.apply_offset(o.clone());
        //has_offset = true;
        //}
        //None => {}
        //};

        if self.index == 0 || has_offset {
            oscillator.update(self);
        }
        oscillator.generate(&self)
    }
}
impl Renderable<Vec<RenderOp>> for Vec<RenderOp> {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);

        let mut iter = self.iter();

        while let Some(op) = iter.next() {
            let stereo_waveform = op.clone().render(oscillator, offset);
            result.append(stereo_waveform);
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
            let (l, r) = point_op_to_gains(&op, &basis);
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
        index: 0,
        samples: (l * 44_100.0).round() as usize,
        total_samples: (l * 44_100.0).round() as usize,
        attack: r_to_f64(point_op.attack * basis.a) * 44_100.0,
        decay: r_to_f64(point_op.decay * basis.d) * 44_100.0,
        osc_type: point_op.osc_type,
        decay_length: point_op.decay_length,
        portamento: r_to_f64(point_op.portamento) as usize,
        voice,
        event,
        next_l_silent,
        next_r_silent,
    };

    *time += point_op.l * basis.l;

    render_op
}

pub fn m_a_and_basis_to_f64(basis: Rational64, m: Rational64, a: Rational64) -> f64 {
    r_to_f64(basis * m) + r_to_f64(a)
}

pub fn calculate_fgpl(basis: &Basis, point_op: &PointOp) -> (f64, (f64, f64), f64, f64) {
    let (f, g) = if point_op.is_silent() {
        (0.0, (0.0, 0.0))
    } else {
        let g = point_op_to_gains(point_op, basis);
        (m_a_and_basis_to_f64(basis.f, point_op.fm, point_op.fa), g)
    };
    let p = m_a_and_basis_to_f64(basis.p, point_op.pm, point_op.pa);
    let l = r_to_f64(point_op.l * basis.l);

    (f, g, p, l)
}

pub fn nf_to_vec_renderable(
    composition: &NormalForm,
    table: &OpOrNfTable,
    basis: &Basis,
) -> Vec<Vec<RenderOp>> {
    let mut normal_form = NormalForm::init();
    composition.apply_to_normal_form(&mut normal_form, table);

    let result: Vec<Vec<RenderOp>> = normal_form
        .operations
        .iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result: Vec<RenderOp> = vec![];
            let iter = vec_point_op.iter();
            for (event, p_op) in iter.enumerate() {
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
            result
        })
        .collect();

    result
}
