extern crate colored;
extern crate num_rational;
use crate::ast::{OpOrNf, OpOrNfTable, OscType, ASR};
use crate::operations::{GetLengthRatio, NameSet, NormalForm, PointOp};
use colored::*;
use num_rational::{Ratio, Rational64};
use std::cmp::Ordering::{Equal, Greater, Less};

pub fn handle_id_error(id: String, table: &OpOrNfTable) -> OpOrNf {
    let result = table.get(&id);

    match result {
        Some(result) => result.clone(),
        None => {
            let id = id;
            println!("Not able to find {} in let table", id.red().bold());
            panic!("Id Not Found");
        }
    }
}

pub fn modulate(input: &[PointOp], modulator: &[PointOp]) -> Vec<PointOp> {
    let mut m = modulator.to_owned();
    let mut i = input.to_owned();
    let mut result = vec![];
    while m.len() > 0 && i.len() > 0 {
        let mut inpu = i[0].clone();
        let mut modu = m[0].clone();
        let modu_l = modu.l;
        let inpu_l = inpu.l;
        if modu_l < inpu_l {
            modu.mod_by(inpu);
            result.push(modu);

            i[0].l -= modu_l;

            m.remove(0);
        } else if modu.l > inpu.l {
            inpu.mod_by(modu);
            result.push(inpu);

            m[0].l -= inpu_l;

            i.remove(0);
        } else {
            inpu.mod_by(modu);
            result.push(inpu);

            i.remove(0);
            m.remove(0);
        }
    }

    result
}

pub fn pad_length(input: &mut NormalForm, max_len: Rational64, table: &OpOrNfTable) {
    let input_lr = input.get_length_ratio(table);
    if max_len > Rational64::new(0, 1) && input_lr < max_len {
        for voice in input.operations.iter_mut() {
            let osc_type = voice.clone().last().unwrap().osc_type;
            voice.push(PointOp {
                fm: Ratio::new(0, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: max_len - input_lr,
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type,
                names: NameSet::new(),
            });
        }
    }
    input.length_ratio = max_len;
}

pub fn join_sequence(mut l: NormalForm, mut r: NormalForm) -> NormalForm {
    if l.operations.is_empty() {
        return r;
    }

    let diff = l.operations.len() as isize - r.operations.len() as isize;
    match diff.partial_cmp(&0).unwrap() {
        Equal => {}
        Greater => {
            for _ in 0..diff {
                r.operations.push(vec![PointOp {
                    fm: Ratio::new(0, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(0, 1),
                    l: r.length_ratio,
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                }])
            }
        }
        Less => {
            for _ in 0..-diff {
                l.operations.push(vec![PointOp {
                    fm: Ratio::new(0, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(0, 1),
                    l: l.length_ratio,
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                }])
            }
        }
    }

    let mut result = NormalForm::init_empty();

    for (left, right) in l.operations.iter_mut().zip(r.operations.iter_mut()) {
        left.append(right);

        result.operations.push(left.clone());
    }

    result.length_ratio += r.length_ratio;
    result.length_ratio += l.length_ratio;

    result
}
