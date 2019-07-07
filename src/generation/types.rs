use crate::instrument::Basis;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

fn denormalize_value(value: f64, min: f64, max: f64, goal_min: f64, goal_max: f64) -> f64 {
    (goal_max - goal_min) / (max - min) * (value - max) + goal_max
    //value * (max - min) + min
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EventType {
    On,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Json1d {
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
        self.x = 2.0 * normalize_value(self.x, normalizer.x.min, normalizer.x.max) - 1.0;
        self.y = normalize_value(self.y, normalizer.y.min, normalizer.y.max);
        self.z = normalize_value(self.z, normalizer.z.min, normalizer.z.max);
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

    pub fn denormalize(&mut self, normalizer: &NormalizerJson) {
        let n = &normalizer.normalizer;
        self.pan = denormalize_value(self.pan, -1.0, 1.0, n.x.min, n.x.max);
        self.frequency = denormalize_value(self.frequency, 0.0, 1.0, n.y.min, n.y.max);
        self.gain = denormalize_value(self.gain, 0.0, 1.0, n.z.min, n.z.max);
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
        let is_silent = (self.fm == zero && self.fa < Rational64::new(40, 1)) || self.g == zero;
        let y = if is_silent {
            0.0
        } else {
            (basis.f * r_to_f64(self.fm)) + r_to_f64(self.fa)
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
            y: y.log10(),
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
