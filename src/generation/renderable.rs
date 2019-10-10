use crate::generation::parsed_to_render::r_to_f64;
use crate::generation::{filename_to_render, RenderReturn, RenderType};
use crate::instrument::oscillator::{point_op_to_gains, Basis};
use crate::instrument::{Oscillator, StereoWaveform};
use num_rational::Rational64;
use socool_ast::{NormalForm, Normalize, OpOrNfTable, OscType, PointOp};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderOp {
    pub f: f64,
    pub p: f64,
    pub g: f64,
    pub l: f64,
    pub t: f64,
    pub attack: f64,
    pub decay: f64,
    pub decay_length: usize,
    pub samples: usize,
    pub voice: usize,
    pub event: usize,
    pub portamento: f64,
    pub osc_type: OscType,
    pub next_l_silent: bool,
    pub next_r_silent: bool,
}

#[allow(dead_code)]
fn pointop_to_renderop(
    point_op: &PointOp,
    time: &mut Rational64,
    voice: usize,
    event: usize,
    basis: &Basis,
    next: Option<PointOp>,
) -> RenderOp {
    let (l_gain, r_gain) = point_op_to_gains(&point_op, &basis);
    let mut next_l_gain = 0.0;
    let mut next_r_gain = 0.0;
    let mut next_silent = false;

    match next {
        Some(op) => {
            let (l, r) = point_op_to_gains(&op, &basis);
            next_l_gain = l;
            next_r_gain = r;
            next_silent = op.is_silent();
        }
        None => {}
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
        samples: (l * 44_100.0).round() as usize,
        attack: r_to_f64(point_op.attack * basis.a),
        osc_type: point_op.osc_type,
        decay: r_to_f64(point_op.decay * basis.a),
        decay_length: point_op.decay_length,
        portamento: r_to_f64(point_op.portamento),
        voice,
        event,
        next_l_silent,
        next_r_silent,
    };

    *time += point_op.l * basis.l;

    render_op
}

fn m_a_and_basis_to_f64(basis: Rational64, m: Rational64, a: Rational64) -> f64 {
    r_to_f64(basis * m) + r_to_f64(a)
}

fn calculate_fgpl(basis: &Basis, point_op: &PointOp) -> (f64, f64, f64, f64) {
    let (f, g) = if point_op.is_silent() {
        (0.0, 0.0)
    } else {
        (
            m_a_and_basis_to_f64(basis.f, point_op.fm, point_op.fa),
            r_to_f64(point_op.g * basis.g),
        )
    };
    let p = m_a_and_basis_to_f64(basis.p, point_op.pm, point_op.pa);
    let l = r_to_f64(point_op.l * basis.l);

    (f, g, p, l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fgpl() {
        let basis = Basis {
            f: Rational64::new(2, 1),
            g: Rational64::new(1, 1),
            p: Rational64::new(0, 1),
            l: Rational64::new(1, 1),
            a: Rational64::new(1, 1),
            d: Rational64::new(1, 1),
        };
        let point_op = PointOp::init();
        let result = calculate_fgpl(&basis, &point_op);
        let expected = (2.0, 1.0, 0.0, 1.0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_m_a_and_basis_to_f64() {
        let result = m_a_and_basis_to_f64(
            Rational64::new(2, 1),
            Rational64::new(300, 1),
            Rational64::new(4, 1),
        );
        let expected = 604.0;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nf_to_vec_renderable() {
        let mut nf = NormalForm::init();
        let (nf, basis, table) = match filename_to_render(
            &"songs/test/render_op.socool".to_string(),
            RenderType::NfBasisAndTable,
        )
        .unwrap()
        {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => {
                panic!("missing songs/tests/render_op.socool");
            }
        };
        //dbg!(nf);
        let result = nf_to_vec_renderable(&nf, &table, &basis);
        let expected: Vec<Vec<RenderOp>> = vec![vec![RenderOp {
            f: 220.0,
            p: 0.0,
            g: 1.0,
            l: 1.0,
            t: 0.0,
            attack: 1.0,
            decay: 1.0,
            decay_length: 2,
            samples: 44100,
            voice: 0,
            event: 0,
            portamento: 1.0,
            osc_type: OscType::Sine,
            next_l_silent: false,
            next_r_silent: false,
        }]];
        assert_eq!(result, expected);
    }
}

#[allow(dead_code)]
pub fn nf_to_vec_renderable(
    composition: &NormalForm,
    table: &OpOrNfTable,
    basis: &Basis,
) -> Vec<Vec<RenderOp>> {
    let mut normal_form = NormalForm::init();
    composition.apply_to_normal_form(&mut normal_form, table);

    let n_voices = normal_form.operations.len();
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

    //result.sort_unstable_by_key(|a| a.t);

    //dbg!(&result);
    result
}

pub trait Renderable<T> {
    fn render<R: Renderable<R>>(
        &mut self,
        basis: &Basis,
        oscillator: &mut Vec<Oscillator>,
        next: Option<R>,
    ) -> StereoWaveform;
}

impl Renderable<RenderOp> for RenderOp {
    fn render<RenderOp>(
        &mut self,
        basis: &Basis,
        oscillator: &mut Vec<Oscillator>,
        next: Option<RenderOp>,
    ) -> StereoWaveform {
        unimplemented!();
    }
}
