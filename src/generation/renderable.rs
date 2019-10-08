use crate::generation::parsed_to_render::r_to_f64;
use crate::instrument::oscillator::{point_op_to_gains, Basis};
use crate::instrument::{Oscillator, StereoWaveform};
use num_rational::Rational64;
use socool_ast::{NormalForm, Normalize, OpOrNfTable, OscType, PointOp};

#[derive(Debug, Clone)]
pub struct RenderOp {
    //TODO: Should be f, p. All values should already reflect basis.
    pub t: f64,
    pub samples: usize,
    pub voice: usize,
    pub event: usize,
    pub attack: f64,
    pub decay: f64,
    pub decay_length: usize,
    pub portamento: f64,
    pub osc_type: OscType,
    pub f: f64,
    pub p: f64,
    pub g: f64,
    pub l: f64,
    pub next_l_silent: bool,
    pub next_r_silent: bool,
}

//pub fn update(basis: Basis, point_op: &PointOp, next_op: Option<PointOp>) {
//let fm = r_to_f64(point_op.fm);
//let fa = r_to_f64(point_op.fa);
//let attack = r_to_f64(point_op.attack);
//let decay = r_to_f64(point_op.decay);

//let (l_gain, r_gain) = point_op_to_gains(&point_op, &basis);
//let mut next_l_gain = 0.0;
//let mut next_r_gain = 0.0;
//let mut next_fm = 0.0;

//match next_op {
//Some(op) => {
//let (l, r) = point_op_to_gains(&op, &basis);
//next_l_gain = l;
//next_r_gain = r;
//next_fm = r_to_f64(op.fm);
//}
//None => {}
//}

//let (ref mut l_voice, ref mut r_voice) = self.voices;

//let silence_next_l = next_fm == 0.0 || next_l_gain == 0.0;
//let silence_next_r = next_fm == 0.0 || next_r_gain == 0.0;

//l_voice.update(VoiceUpdate {
//frequency: (r_to_f64(basis.f) * fm) + fa,
//gain: l_gain,
//osc_type: point_op.osc_type,
//silence_next: silence_next_l,
//attack: basis.a * attack,
//decay: basis.d * decay,
//decay_type: point_op.decay_length,
//});
//r_voice.update(VoiceUpdate {
//frequency: (r_to_f64(basis.f) * fm) + fa,
//gain: r_gain,
//osc_type: point_op.osc_type,
//silence_next: silence_next_r,
//attack: basis.a * attack,
//decay: basis.d * decay,
//decay_type: point_op.decay_length,
//});
//}

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

    let l = r_to_f64(point_op.l * basis.l);
    let next_l_silent = next_silent || next_l_gain == 0.0;
    let next_r_silent = next_silent || next_r_gain == 0.0;

    let render_op = RenderOp {
        f: r_to_f64(basis.f * point_op.fm) + r_to_f64(point_op.fa),
        p: r_to_f64(basis.p * point_op.pm) + r_to_f64(point_op.pa),
        g: r_to_f64(point_op.g * basis.g),
        l,
        t: r_to_f64(time.clone()),
        samples: (l * 44_100.0).round() as usize,
        attack: r_to_f64(point_op.attack) * basis.a,
        osc_type: point_op.osc_type,
        decay: r_to_f64(point_op.decay) * basis.a,
        decay_length: point_op.decay_length,
        portamento: r_to_f64(point_op.portamento),
        voice,
        event,
        next_l_silent,
        next_r_silent,
    };

    *time += point_op.l;

    render_op
}

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
        StereoWaveform::new(1024)
    }
}
