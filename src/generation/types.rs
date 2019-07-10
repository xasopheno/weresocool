use crate::instrument::Basis;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use socool_ast::{NormalForm, Op, OpOrNf, PointOp, NameSet, OscType};

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

fn normalize_value(value: f64, min: f64, max: f64, goal_min: f64, goal_max: f64) -> f64 {
    (goal_max - goal_min) / (max - min) * (value - max) + goal_max
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EventType {
    On,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Json1d {
    pub filename: String,
    pub ops: Vec<Op4D>,
    pub length: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizerJson {
    pub filename: String,
    pub normalizer: Normalizer,
    pub basis: Basis,
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

impl Op4D {
    pub fn normalize(&mut self, normalizer: &Normalizer) {
        self.x = normalize_value(self.x, normalizer.x.min, normalizer.x.max, 0.0, 1.0);
        self.y = normalize_value(self.y, normalizer.y.min, normalizer.y.max, 0.0, 1.0);
        self.z = normalize_value(self.z, normalizer.z.min, normalizer.z.max, 0.0, 1.0);
    }

    pub fn to_op_csv_1d(&self) -> OpCsv1d {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpCsv1d {
    pub time: f64,
    pub length: f64,
    pub frequency: f64,
    pub pan: f64,
    pub gain: f64,
    pub voice: usize,
    pub event: usize,
}

impl OpCsv1d {
    pub fn to_op4d(&self) -> Op4D {
        Op4D {
            t: self.time,
            event_type: EventType::On,
            voice: self.voice,
            event: self.event,
            x: self.pan,
            y: self.frequency,
            z: self.gain,
            l: self.length,
        }
    }

    pub fn to_point_op(&self) -> PointOp {
        let fm = Rational64::approximate_float(self.frequency).unwrap();
        let pa = Rational64::approximate_float(self.pan).unwrap();
        let g = Rational64::approximate_float(self.gain).unwrap();
        let l = Rational64::approximate_float(self.length).unwrap();
        PointOp {
            fm,
            fa: Rational64::new(0, 1),
            pm: Rational64::new(0, 1),
            pa,
            g,
            l,
            attack: Rational64::new(1, 1),
            decay: Rational64::new(1, 1),
            decay_length: 2,
            portamento: Rational64::new(1, 1),
            osc_type: OscType::Sine,
            names: NameSet::new(),
        }
    }

    pub fn denormalize(&mut self, normalizer: &NormalizerJson) {
        let n = &normalizer.normalizer;
        self.pan = normalize_value(self.pan, -1.0, 1.0, n.x.min, n.x.max);
        self.frequency = normalize_value(self.frequency, 0.0, 1.0, n.y.min, n.y.max);
        self.gain = normalize_value(self.gain, 0.0, 1.0, n.z.min, n.z.max);
    }
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
    pub fn to_op_4d(&self, basis: &Basis) -> Op4D {
        let zero = Rational64::new(0, 1);
        let is_silent = (self.fm == zero && self.fa < Rational64::new(20, 1)) || self.g == zero;
        let y = if is_silent {
            0.0
        } else {
            ((basis.f * r_to_f64(self.fm)) + r_to_f64(self.fa)).log10()
        };
        let z = if is_silent {
            0.0
        } else {
            basis.g * r_to_f64(self.g)
        };
        Op4D {
            l: r_to_f64(self.l) * basis.l,
            t: r_to_f64(self.t) * basis.l,
            x: ((basis.p + r_to_f64(self.pa)) * r_to_f64(self.pm)),
            y,
            z,
            voice: self.voice,
            event: self.event,
            event_type: self.event_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Normalizer {
    pub x: MinMax,
    pub y: MinMax,
    pub z: MinMax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

//#[cfg(test)]
//mod test_types {
//use super::*;
#[test]
fn test_timed_op_to_op4d() {
    let mut t_op = TimedOp {
        t: Rational64::new(1, 1),
        fm: Rational64::new(1, 1),
        fa: Rational64::new(0, 1),
        pm: Rational64::new(1, 1),
        pa: Rational64::new(0, 1),
        g: Rational64::new(1, 1),
        l: Rational64::new(1, 1),
        event_type: EventType::On,
        voice: 1,
        event: 1,
    };

    let basis = Basis {
        f: 100.0,
        g: 1.0,
        p: 0.0,
        l: 1.0,
        a: 44100.0,
        d: 44100.0,
    };

    let mut expected = Op4D {
        l: 1.0,
        t: 1.0,
        x: 0.0,
        y: 2.0,
        z: 1.0,
        voice: 1,
        event: 1,
        event_type: EventType::On,
    };
    let result = t_op.to_op_4d(&basis);
    assert_eq!(&result, &expected);

    t_op.fm = Rational64::new(0, 1);
    let result = t_op.to_op_4d(&basis);

    expected.y = 0.0;
    expected.z = 0.0;
    assert_eq!(&result, &expected);

    t_op.g = Rational64::new(0, 1);
    t_op.fm = Rational64::new(1, 1);
    t_op.fa = Rational64::new(10, 1);
    let result = t_op.to_op_4d(&basis);

    expected.y = 0.0;
    expected.z = 0.0;
    assert_eq!(&result, &expected);

    t_op.g = Rational64::new(1, 1);
    t_op.fa = Rational64::new(21, 1);
    let result = t_op.to_op_4d(&basis);

    expected.y = 2.0827853703164503;
    expected.z = 1.0;
    assert_eq!(&result, &expected);
}

#[test]
fn test_normalize_denormalize_value() {
    let input = 5.0;
    let result_norm = normalize_value(input, 0.0, 10.0, 0.0, 1.0);
    assert_eq!(result_norm, 0.5);

    let result_denorm = normalize_value(result_norm, 0.0, 1.0, 0.0, 10.0);
    assert_eq!(result_denorm, 5.0);

    let input = 2.5;
    let result_norm = normalize_value(input, -5.0, 5.0, 0.0, 1.0);
    assert_eq!(result_norm, 0.75);

    let result_denorm = normalize_value(result_norm, 0.0, 1.0, -5.0, 5.0);
    assert_eq!(result_denorm, 2.5);
}
