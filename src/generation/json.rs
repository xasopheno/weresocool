use crate::{
    generation::nn_data_generation::{CSVOp, Normalizer},
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

fn normalize_value(value: Rational64, min: Rational64, max: Rational64) -> f64 {
    let result = (value - min) / (max - min);
    r_to_f64(result)
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

    //    pub fn to_csv_op(&self) -> CSVOp {
    //        CSVOp {
    //            fm: r_to_f64(self.fm),
    //            fa: r_to_f64(self.fa),
    //            pm: r_to_f64(self.pm),
    //            pa: r_to_f64(self.pa),
    //            g: r_to_f64(self.g),
    //            l: r_to_f64(self.l),
    //            v: self.voice,
    //        }
    //    }

    pub fn to_csv_op(&self) -> CSVOp {
        let zero = Rational64::new(0, 1);
        let is_silent = self.fm == Rational64::new(0, 1) || self.g == Rational64::new(0, 1);
        let fm = if is_silent { zero } else { self.fm };
        let g = if is_silent { zero } else { self.g };

        CSVOp {
            fm: r_to_f64(fm),
            fa: r_to_f64(self.fa),
            pm: r_to_f64(self.pm),
            pa: r_to_f64(self.pa),
            g: r_to_f64(g),
            l: r_to_f64(self.l),
            v: self.voice as usize,
        }
    }

    //    pub fn to_normalized_csv_op(&self, normalizer: Normalizer) -> CSVOp {
    //        let zero = Rational64::new(0, 1);
    //        let is_silent = self.fm == Rational64::new(0, 1) || self.g == Rational64::new(0, 1);
    //        let fm = if is_silent { zero } else { self.fm };
    //        let g = if is_silent { zero } else { self.g };
    //
    //        CSVOp {
    //            fm: normalize_value(fm, normalizer.fm.0, normalizer.fm.1),
    //            fa: normalize_value(self.fa, normalizer.fa.0, normalizer.fa.1),
    //            pm: normalize_value(self.pm, normalizer.pm.0, normalizer.pm.1),
    //            pa: normalize_value(self.pa, normalizer.pa.0, normalizer.pa.1),
    //            g: normalize_value(g, normalizer.g.0, normalizer.g.1),
    //            l: normalize_value(self.l, normalizer.l.0, normalizer.l.1),
    //            v: normalize_value(
    //                Rational64::new(self.voice as i64, 1),
    //                normalizer.v.0,
    //                normalizer.v.1,
    //            ) as usize,
    //        }
    //    }
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

pub fn normalform_to_vec_timed_op_1d(normalform: &NormalForm, table: &OpOrNfTable) -> Vec<TimedOp> {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    normalform.apply_to_normal_form(&mut normal_form, table);

    let mut result: Vec<TimedOp> = normal_form
        .operations
        .iter()
        .enumerate()
        .flat_map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result = vec![];
            vec_point_op.iter().enumerate().for_each(|(event, p_op)| {
                let (on, _off) = point_op_to_timed_op(p_op, &mut time, voice, event);
                result.push(on);
                //                result.push(off);
            });
            result
        })
        .collect();

    result.sort_unstable_by_key(|a| a.t);

    result
}

pub fn normalform_to_vec_timed_op_2d(
    normalform: &NormalForm,
    table: &OpOrNfTable,
) -> Vec<Vec<TimedOp>> {
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    normalform.apply_to_normal_form(&mut normal_form, table);

    let result: Vec<Vec<TimedOp>> = normal_form
        .operations
        .iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
            let mut time = Rational64::new(0, 1);
            let mut result = vec![];
            vec_point_op.iter().enumerate().for_each(|(event, p_op)| {
                let (on, _) = point_op_to_timed_op(p_op, &mut time, voice, event);
                result.push(on);
            });
            result
        })
        .collect();

    result
}

pub fn to_json(basis: &Basis, composition: &NormalForm, table: &OpOrNfTable, filename: String) {
    banner("JSONIFY-ing".to_string(), filename.clone());

    let vec_timed_op = normalform_to_vec_timed_op_1d(composition, table);
    let vec_op4d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    let json = to_string(&vec_op4d).unwrap();

    write_composition_to_json(&json, &filename).expect("Writing to JSON failed");
    printed("JSON".to_string());
}
