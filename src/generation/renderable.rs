use socool_ast::{OscType, PointOp, OpOrNfTable, NormalForm, Normalize};
use crate::instrument::{StereoWaveform, Oscillator};
use crate::instrument::oscillator::Basis;
use crate::generation::{parsed_to_render::r_to_f64};
use num_rational::Rational64;


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
    pub fm: f64,
    pub fa: f64,
    pub pm: f64,
    pub pa: f64,
    pub g: f64,
    pub l: f64,
    pub next: Option<PointOp>,
}

fn pointop_to_renderop(
    point_op: &PointOp,
    time: &mut Rational64,
    voice: usize,
    event: usize,
    basis: &Basis,
    next: Option<PointOp>,
) -> RenderOp {
    let render_op = RenderOp {
        fm: r_to_f64(point_op.fm),
        fa: r_to_f64(point_op.fa),
        pm: r_to_f64(point_op.pm),
        pa: r_to_f64(point_op.pa),
        samples: (r_to_f64(point_op.l) * 44_100.0).round() as usize,
        attack: r_to_f64(point_op.attack),
        osc_type: point_op.osc_type,
        decay: r_to_f64(point_op.decay),
        decay_length: point_op.decay_length,
        portamento: r_to_f64(point_op.portamento),
        g: r_to_f64(point_op.g),
        l: r_to_f64(point_op.l),
        t: r_to_f64(time.clone()),
        voice,
        event,
        next: next
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
    let mut result: Vec<Vec<RenderOp>> = normal_form
        .operations
        .iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result: Vec<RenderOp> = vec![];
            let mut iter = vec_point_op.iter();
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


