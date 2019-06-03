use crate::{
    instrument::Basis,
    ui::{banner, printed},
    write::write_composition_to_json,
};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use socool_ast::{NormalForm, Normalize, OpOrNfTable, PointOp};

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimedOp {
    pub t: Rational64,
    pub event_type: EventType,
    pub voice: usize,
    pub event: usize,
    pub fm: Rational64,
    pub fa: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub g: Rational64,
    pub l: Rational64,
}

impl TimedOp {
    fn to_op_4d(&self, basis: &Basis) -> Op4D {
        Op4D {
            l: r_to_f64(self.l) * basis.l,
            t: r_to_f64(self.t) * basis.l,
            x: ((basis.p + r_to_f64(self.pa)) * r_to_f64(self.pm)),
            y: (basis.f * r_to_f64(self.fm)) + r_to_f64(self.fa),
            z: basis.g * r_to_f64(self.g),
            voice: self.voice,
            event: self.event,
            event_type: self.event_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Op4D {
    pub t: f64,
    pub event_type: EventType,
    pub voice: usize,
    pub event: usize,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub l: f64,
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
        let is_silent = self.y == 0.0 || self.z == 0.0;
        //        let y = if is_silent { 0.0 } else { self.y };
        let z = if is_silent { 0.0 } else { self.z };

        self.x = normalize_value(self.x, normalizer.x.min, normalizer.x.max) - 0.5;
        //        dbg!(normalize_value(self.x, normalizer.x.min, normalizer.x.max) - 0.5);
        //        self.y = normalize_value(y, normalizer.y.min, normalizer.z.max);
        self.z = normalize_value(z, normalizer.z.min, normalizer.z.max);
    }
}

fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

fn normalize_op4d_1d(op4d_1d: &mut Vec<Op4D>, n: Normalizer) {
    op4d_1d.iter_mut().for_each(|op| {
        op.normalize(&n);
    })
}

fn get_min_max_op4d_1d(vec_op4d: &Vec<Op4D>) -> Normalizer {
    let mut max_state = Op4D {
        t: 0.0,
        event: 0,
        event_type: EventType::On,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
    };

    let mut min_state = Op4D {
        t: 0.0,
        event: 0,
        event_type: EventType::On,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
    };

    for op in vec_op4d {
        max_state = Op4D {
            x: max_state.x.max((op.x).abs()),
            y: max_state.y.max(op.y),
            z: max_state.z.max(op.z),
            l: max_state.l.max(op.l),
            t: max_state.t.max(op.t),
            event: max_state.event.max(op.event),
            voice: max_state.voice.max(op.voice),
            event_type: EventType::On,
        };
        min_state = Op4D {
            x: min_state.x.min(-(op.x).abs()),
            y: min_state.y.min(op.y),
            z: min_state.z.min(op.z),
            l: min_state.l.min(op.l),
            t: min_state.t.min(op.t),
            event: min_state.event.min(op.event),
            voice: min_state.voice.min(op.voice),
            event_type: EventType::On,
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

    n
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
) -> (TimedOp, TimedOp) {
    let on = TimedOp {
        fm: point_op.fm,
        fa: point_op.fa,
        pm: point_op.pm,
        pa: point_op.pa,
        g: point_op.g,
        l: point_op.l,
        t: time.clone(),
        event_type: EventType::On,
        voice,
        event,
    };

    *time += point_op.l;

    let off = TimedOp {
        t: time.clone(),
        event_type: EventType::Off,
        ..on
    };

    (on, off)
}

pub fn vec_timed_op_to_vec_op4d(timed_ops: Vec<TimedOp>, basis: &Basis) -> Vec<Op4D> {
    timed_ops.iter().map(|t_op| t_op.to_op_4d(&basis)).collect()
}

pub fn composition_to_vec_timed_op(composition: &NormalForm, table: &OpOrNfTable) -> Vec<TimedOp> {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form, table);

    let mut result: Vec<TimedOp> = normal_form
        .operations
        .iter()
        .enumerate()
        .flat_map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result = vec![];
            vec_point_op.iter().enumerate().for_each(|(event, p_op)| {
                let (on, off) = point_op_to_timed_op(p_op, &mut time, voice, event);
                result.push(on);
                result.push(off);
            });
            result
        })
        .collect();

    result.sort_unstable_by_key(|a| a.t);

    result
}

pub fn to_json(basis: &Basis, composition: &NormalForm, table: &OpOrNfTable, filename: String) {
    banner("JSONIFY-ing".to_string(), filename.clone());

    let vec_timed_op = composition_to_vec_timed_op(composition, table);
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    let normalizer = get_min_max_op4d_1d(&op4d_1d);
    normalize_op4d_1d(&mut op4d_1d, normalizer);

    let json = to_string(&op4d_1d).unwrap();

    write_composition_to_json(&json, &filename).expect("Writing to JSON failed");
    printed("JSON".to_string());
}
