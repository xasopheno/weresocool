use super::{Normalizer, OpCSV};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Op4D {
    pub t: f64,
    pub voice: usize,
    pub event: usize,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub l: f64,
    pub names: Vec<String>,
}

impl std::default::Default for Op4D {
    fn default() -> Self {
        Self {
            t: 0.0,
            l: 0.0,
            y: 0.0,
            x: 0.0,
            z: 0.0,
            voice: 0,
            event: 0,
            names: vec![],
        }
    }
}

impl Op4D {
    pub fn normalize(&mut self, normalizer: &Normalizer) {
        self.x = 2.0 * normalize_value(self.x, normalizer.x.min, normalizer.x.max) - 1.0;
        self.y = normalize_value(self.y, normalizer.y.min, normalizer.y.max);
        self.z = normalize_value(self.z, normalizer.z.min, normalizer.z.max);
    }

    pub const fn to_op_csv(&self) -> OpCSV {
        OpCSV {
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

pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    // equivilance check for floats. max == min.
    let d = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        max - min
    };
    let r = (value - min) / d;

    if f64::is_nan(r) {
        0.0
    } else {
        r
    }
}

pub fn normalize_op4d_1d(op4d_1d: &mut [Op4D]) {
    let n = Normalizer::default();
    op4d_1d.iter_mut().for_each(|op| {
        op.normalize(&n);
    })
}
