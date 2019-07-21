use crate::{
    generation::{NNInput, EventType, Json1d, MinMax, Normalizer, NormalizerJson, Op4D, TimedOp}, instrument::Basis,
    ui::{banner, printed},
    write::{write_composition_to_csv, write_composition_to_json, write_normalizer_to_json},
};
use num_rational::Rational64;
//use serde::{Serialize, Deserialize};
use serde_json::to_string;
use socool_ast::{NormalForm, Normalize, OpOrNfTable, PointOp};
pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

fn normalize_op4d_1d(op4d_1d: &mut Vec<Op4D>, n: Normalizer) {
    op4d_1d.iter_mut().for_each(|op| {
        op.normalize(&n);
    })
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct NNNormalizer {
   pub fm: NNMinMax, 
   pub fa: NNMinMax, 
   pub pm: NNMinMax, 
   pub pa: NNMinMax, 
   pub g: NNMinMax, 
   pub l: NNMinMax, 
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct NNMinMax {
    pub max: Rational64,
    pub min: Rational64
}

fn get_min_max_nninput_1d(vec_nn_input: &Vec<NNInput>) -> (NNNormalizer) {
    let mut max_state = NNInput {
        fm: Rational64::new(0,1),
        fa: Rational64::new(0,1),
        pm: Rational64::new(0,1),
        pa: Rational64::new(0,1),
        g: Rational64::new(0,1),
        l: Rational64::new(0,1),
        voice: 0
    };

    let mut min_state = NNInput {
        fm: Rational64::new(0,1),
        fa: Rational64::new(0,1),
        pm: Rational64::new(0,1),
        pa: Rational64::new(0,1),
        g: Rational64::new(0,1),
        l: Rational64::new(0,1),
        voice: 0
    };


    for op in vec_nn_input {

        max_state = NNInput {
            fm: max_state.fm.max(op.fm),
            fa: max_state.fa.max(op.fa),
            pm: max_state.pm.max(op.pm),
            pa: max_state.pa.max(op.pa),
            g: max_state.g.max(op.g),
            l: max_state.l.max(op.l),
            voice: max_state.voice.max(op.voice),
        };

        max_state = NNInput {
            fm: min_state.fm.min(op.fm),
            fa: min_state.fa.min(op.fa),
            pm: min_state.pm.min(op.pm),
            pa: min_state.pa.min(op.pa),
            g: min_state.g.min(op.g),
            l: min_state.l.min(op.l),
            voice: min_state.voice.min(op.voice),
        };
    }

    let n = NNNormalizer {
        fm: NNMinMax {
            min: min_state.fm,
            max: max_state.fm,
        },
        fa: NNMinMax {
            min: min_state.fa,
            max: max_state.fa,
        },
        pm: NNMinMax {
            min: min_state.pm,
            max: max_state.pm,
        },
        pa: NNMinMax {
            min: min_state.pa,
            max: max_state.pa,
        },
        g: NNMinMax {
            min: min_state.g,
            max: max_state.g,
        },
        l: NNMinMax {
            min: min_state.l,
            max: max_state.l,
        },
    };
    dbg!(n.clone());
    n
}

fn get_min_max_op4d_1d(vec_op4d: &Vec<Op4D>) -> (Normalizer, f64) {
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
        event: 10,
        event_type: EventType::On,
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
    dbg!(n.clone());
    dbg!(max_len);
    (n, max_len)
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

pub fn vec_timed_op_to_vec_nninput(timed_ops: Vec<TimedOp>) -> Vec<NNInput> {
    timed_ops.iter().map(|t_op| t_op.to_nninput()).collect()
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
                let (on, _) = point_op_to_timed_op(p_op, &mut time, voice, event);
                result.push(on);
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

    op4d_1d.retain(|op| {
        let is_silent = op.y == 0.0 || op.z <= 0.0;
        !is_silent
    });

    let (normalizer, max_len) = get_min_max_op4d_1d(&op4d_1d);

    normalize_op4d_1d(&mut op4d_1d, normalizer.clone());

    let json = to_string(&Json1d {
        filename: filename.clone(),
        ops: op4d_1d,
        length: max_len,
    })
    .unwrap();

    write_composition_to_json(&json, &filename).expect("Writing to JSON failed");
    printed("JSON".to_string());
}

pub fn to_csv(basis: &Basis, composition: &NormalForm, table: &OpOrNfTable, filename: String) {
    banner("CSV-ing".to_string(), filename.clone());
    let vec_timed_op = composition_to_vec_timed_op(composition, table);
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    op4d_1d.iter_mut().for_each(|op| {
        let is_silent = op.y == 0.0 || op.z <= 0.0;

        if is_silent {
            op.y = 0.0;
            op.z = 0.0;
        };
    });

    let (normalizer, _max_len) = get_min_max_op4d_1d(&op4d_1d);

    let normalizer_string = to_string(&NormalizerJson {
        filename: filename.clone(),
        normalizer: normalizer.clone(),
        basis: basis.clone(),
    })
    .unwrap();

    write_normalizer_to_json(&normalizer_string, &filename.clone());

    normalize_op4d_1d(&mut op4d_1d, normalizer);

    write_composition_to_csv(&mut op4d_1d, &filename);
}
