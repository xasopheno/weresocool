use crate::{
    ui::{banner, printed},
    write::{write_composition_to_csv, write_composition_to_json},
};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use weresocool_ast::{Defs, NameSet, NormalForm, Normalize, OscType, PointOp, ASR};
use weresocool_error::Error;
use weresocool_instrument::Basis;

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimedOp {
    pub t: Rational64,
    pub event_type: EventType,
    pub voice: usize,
    pub event: usize,
    pub attack: Rational64,
    pub decay: Rational64,
    pub asr: ASR,
    pub portamento: Rational64,
    pub osc_type: OscType,
    pub fm: Rational64,
    pub fa: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub g: Rational64,
    pub l: Rational64,
}

impl TimedOp {
    fn to_op_4d(&self, basis: &Basis) -> Op4D {
        let zero = Rational64::new(0, 1);
        let is_silent = (self.fm == zero && self.fa < Rational64::new(20, 1)) || self.g == zero;
        let y = if is_silent {
            0.0
        } else {
            r_to_f64(basis.f).mul_add(r_to_f64(self.fm), r_to_f64(self.fa))
        };
        let z = if is_silent {
            0.0
        } else {
            r_to_f64(basis.g) * r_to_f64(self.g)
        };
        Op4D {
            l: r_to_f64(self.l) * r_to_f64(basis.l),
            t: r_to_f64(self.t) * r_to_f64(basis.l),
            x: ((r_to_f64(basis.p) + r_to_f64(self.pa)) * r_to_f64(self.pm)),
            y: y.log10(),
            z,
            voice: self.voice,
            event: self.event,
            // event_type: self.event_type.clone(),
        }
    }
    #[allow(clippy::missing_const_for_fn)]
    pub fn to_point_op(&self) -> PointOp {
        PointOp {
            fm: self.fm,
            fa: self.fa,
            pm: self.pm,
            pa: self.pa,
            g: self.g,
            l: self.l,
            attack: self.decay,
            decay: self.decay,
            asr: self.asr,
            portamento: self.portamento,
            osc_type: self.osc_type,
            names: NameSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Op4D {
    pub t: f64,
    // pub event_type: EventType,
    pub voice: usize,
    pub event: usize,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub l: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpCsv1d {
    time: f64,
    length: f64,
    frequency: f64,
    pan: f64,
    gain: f64,
    voice: usize,
    event: usize,
}

#[derive(Debug, Clone)]
pub struct Normalizer {
    pub x: MinMax,
    pub y: MinMax,
    pub z: MinMax,
}

#[derive(Debug, Clone)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

impl Op4D {
    pub fn normalize(&mut self, normalizer: &Normalizer) {
        self.x = 2.0 * normalize_value(self.x, normalizer.x.min, normalizer.x.max) - 1.0;
        self.y = normalize_value(self.y, normalizer.y.min, normalizer.y.max);
        self.z = normalize_value(self.z, normalizer.z.min, normalizer.z.max);
    }

    pub const fn to_op_csv_1d(&self) -> OpCsv1d {
        OpCsv1d {
            time: self.t,
            length: self.l,
            frequency: self.y,
            pan: self.x,
            gain: self.z,
            voice: self.voice,
            event: self.event,
        }
    }
}

fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    // equivilance check for floats. max == min.
    let d = if (max - min).abs() < std::f64::EPSILON {
        1.0
    } else {
        max - min
    };
    (value - min) / d
}

fn normalize_op4d_1d(op4d_1d: &mut Vec<Op4D>, n: Normalizer) {
    op4d_1d.iter_mut().for_each(|op| {
        op.normalize(&n);
    })
}

fn get_min_max_op4d_1d(vec_op4d: &[Op4D]) -> (Normalizer, f64) {
    let mut max_state = Op4D {
        t: 0.0,
        event: 0,
        // event_type: EventType::On,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
    };

    let mut min_state = Op4D {
        t: 0.0,
        event: 10,
        // event_type: EventType::On,
        voice: 10,
        x: 0.0,
        y: 10_000.0,
        z: 1.0,
        l: 1.0,
    };

    let mut max_len: f64 = 0.0;
    for op in vec_op4d {
        max_len = max_len.max(op.t + op.l);

        max_state = Op4D {
            x: max_state.x.max((op.x).abs()),
            y: max_state.y.max(op.y),
            z: max_state.z.max(op.z),
            l: max_state.l.max(op.l),
            t: max_state.t.max(op.t),
            event: max_state.event.max(op.event),
            voice: max_state.voice.max(op.voice),
            // event_type: EventType::On,
        };

        min_state = Op4D {
            x: min_state.x.min(-(op.x).abs()),
            y: min_state.y.min(op.y),
            z: min_state.z.min(op.z),
            l: min_state.l.min(op.l),
            t: min_state.t.min(op.t),
            event: min_state.event.min(op.event),
            voice: min_state.voice.min(op.voice),
            // event_type: EventType::On,
        };
    }

    let n = Normalizer {
        x: MinMax {
            min: min_state.x,
            max: max_state.x,
        },
        y: MinMax {
            min: min_state.y,
            max: max_state.y,
        },
        z: MinMax {
            min: min_state.z,
            max: max_state.z,
        },
    };
    dbg!(n.clone());
    dbg!(max_len);
    (n, max_len)
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EventType {
    On,
    Off,
}

fn point_op_to_timed_op(
    point_op: &PointOp,
    time: &mut Rational64,
    voice: usize,
    event: usize,
) -> TimedOp {
    let timed_op = TimedOp {
        fm: point_op.fm,
        fa: point_op.fa,
        pm: point_op.pm,
        pa: point_op.pa,
        attack: point_op.attack,
        osc_type: point_op.osc_type,
        decay: point_op.decay,
        asr: point_op.asr,
        portamento: point_op.portamento,
        g: point_op.g,
        l: point_op.l,
        t: *time,
        event_type: EventType::On,
        voice,
        event,
    };

    *time += point_op.l;

    timed_op
}

pub fn vec_timed_op_to_vec_op4d(timed_ops: Vec<TimedOp>, basis: &Basis) -> Vec<Op4D> {
    timed_ops.iter().map(|t_op| t_op.to_op_4d(basis)).collect()
}

pub fn composition_to_vec_timed_op(
    composition: &NormalForm,
    defs: &Defs,
) -> Result<(Vec<TimedOp>, usize), Error> {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form, defs)?;

    let n_voices = normal_form.operations.len();
    let mut result: Vec<TimedOp> = normal_form
        .operations
        .iter()
        .enumerate()
        .flat_map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result = vec![];
            let iter = vec_point_op.iter();
            for (event, p_op) in iter.enumerate() {
                let op = point_op_to_timed_op(p_op, &mut time, voice, event);
                result.push(op);
            }
            result
        })
        .collect();

    result.sort_unstable_by_key(|a| a.t);

    Ok((result, n_voices))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Json1d {
    ops: Vec<Op4D>,
    length: f64,
}

pub fn to_json(
    basis: &Basis,
    composition: &NormalForm,
    defs: &Defs,
    filename: String,
) -> Result<(), Error> {
    banner("JSONIFY-ing".to_string(), filename.clone());

    let (vec_timed_op, _) = composition_to_vec_timed_op(composition, defs)?;
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    //TODO: Factor out
    op4d_1d.retain(|op| {
        let is_silent = op.y == 0.0 || op.z <= 0.0;
        !is_silent
    });

    let (normalizer, max_len) = get_min_max_op4d_1d(&op4d_1d);

    normalize_op4d_1d(&mut op4d_1d, normalizer);

    let json = to_string(&Json1d {
        ops: op4d_1d,
        length: max_len,
    })?;

    write_composition_to_json(&json, &filename)?;
    printed("JSON".to_string());

    Ok(())
}

pub fn to_csv(
    basis: &Basis,
    composition: &NormalForm,
    defs: &Defs,
    filename: String,
) -> Result<(), Error> {
    banner("CSV-ing".to_string(), filename.clone());

    let (vec_timed_op, _) = composition_to_vec_timed_op(composition, defs)?;
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);
    dbg!(&op4d_1d);

    // op4d_1d.retain(|op| {
    // let is_silent = op.y == 0.0 || op.z <= 0.0;
    // !is_silent
    // });

    // let (normalizer, _max_len) = get_min_max_op4d_1d(&op4d_1d);

    // normalize_op4d_1d(&mut op4d_1d, normalizer);

    // write_composition_to_csv(&mut op4d_1d, &filename)?;

    Ok(())
}
