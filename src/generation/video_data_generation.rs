use crate::instrument::oscillator::Basis;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use socool_ast::ast::OpOrNfTable;
use socool_ast::operations::{NormalForm, Normalize as NormalizeOp, PointOp};

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

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EventType {
    On,
    Off,
}

impl TimedOp {
    pub fn to_op_4d(&self, basis: &Basis) -> Op4D {
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

pub fn point_op_to_timed_op(
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
