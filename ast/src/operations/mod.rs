use crate::{Defs, OscType, Term, ASR};
use num_rational::{Ratio, Rational64};
use std::{
    collections::{BTreeSet, HashMap},
    ops::{Mul, MulAssign},
};
use weresocool_error::Error;
mod get_length_ratio;
pub mod helpers;
mod normalize;
pub mod substitute;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct NormalForm {
    pub operations: Vec<Vec<PointOp>>,
    pub length_ratio: Rational64,
}

pub type ArgMap = HashMap<String, Term>;

pub type NameSet = BTreeSet<String>;

#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct PointOp {
    pub fm: Rational64,
    pub fa: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub g: Rational64,
    pub reverb: Rational64,
    pub l: Rational64,
    pub attack: Rational64,
    pub decay: Rational64,
    pub asr: ASR,
    pub portamento: Rational64,
    pub osc_type: OscType,
    pub names: NameSet,
}

pub trait Normalize {
    fn apply_to_normal_form(&self, normal_form: &mut NormalForm, defs: &Defs) -> Result<(), Error>;
}

pub trait GetLengthRatio {
    fn get_length_ratio(&self, defs: &Defs) -> Result<Rational64, Error>;
}

pub trait Substitute {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &Defs,
        arg_map: &ArgMap,
    ) -> Result<Term, Error>;
}

impl GetLengthRatio for NormalForm {
    fn get_length_ratio(&self, _defs: &Defs) -> Result<Rational64, Error> {
        Ok(self.length_ratio)
    }
}

impl Substitute for NormalForm {
    fn substitute(
        &self,
        _normal_form: &mut NormalForm,
        _defs: &Defs,
        _arg_map: &ArgMap,
    ) -> Result<Term, Error> {
        Ok(Term::Nf(self.clone()))
    }
}

pub fn union_names(b_tree_set: NameSet, left: &NameSet) -> NameSet {
    let mut result = b_tree_set;
    for val in left {
        result.insert(val.clone());
    }

    result
}

impl Mul<NormalForm> for NormalForm {
    type Output = NormalForm;

    fn mul(self, other: NormalForm) -> NormalForm {
        let mut nf_result = vec![];
        let mut max_lr = Rational64::new(0, 1);
        for other_seq in self.operations.iter() {
            for self_seq in other.operations.iter() {
                for other_point_op in other_seq.iter() {
                    let mut seq_result: Vec<PointOp> = vec![];
                    let mut seq_lr = Rational64::new(0, 1);
                    for self_point_op in self_seq.iter() {
                        seq_lr += self_point_op.l * other_point_op.l;
                        seq_result.push(other_point_op * self_point_op);
                    }

                    if seq_lr > max_lr {
                        max_lr = seq_lr
                    }

                    nf_result.push(seq_result);
                }
            }
        }

        NormalForm {
            operations: nf_result,
            length_ratio: max_lr,
        }
    }
}

impl MulAssign<&NormalForm> for NormalForm {
    fn mul_assign(&mut self, other: &NormalForm) {
        let mut nf_result = vec![];
        let mut max_lr = Rational64::new(0, 1);
        for other_seq in self.operations.iter() {
            for self_seq in other.operations.iter() {
                for other_point_op in other_seq.iter() {
                    let mut seq_result: Vec<PointOp> = vec![];
                    let mut seq_lr = Rational64::new(0, 1);
                    for self_point_op in self_seq.iter() {
                        seq_lr += self_point_op.l * other_point_op.l;
                        seq_result.push(other_point_op * self_point_op);
                    }

                    if seq_lr > max_lr {
                        max_lr = seq_lr
                    }

                    nf_result.push(seq_result);
                }
            }
        }

        *self = NormalForm {
            operations: nf_result,
            length_ratio: max_lr,
        }
    }
}

impl Normalize for NormalForm {
    fn apply_to_normal_form(&self, input: &mut NormalForm, _defs: &Defs) -> Result<(), Error> {
        *input *= self;
        Ok(())
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Mul<PointOp> for PointOp {
    type Output = PointOp;

    fn mul(self, other: PointOp) -> PointOp {
        let names = union_names(self.names.clone(), &other.names);
        PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l * other.l,
            reverb: other.reverb,
            osc_type: other.osc_type,
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<'a, 'b> Mul<&'b PointOp> for &'a PointOp {
    type Output = PointOp;

    fn mul(self, other: &'b PointOp) -> PointOp {
        let names = union_names(self.names.clone(), &other.names);
        PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l * other.l,
            reverb: other.reverb,
            osc_type: other.osc_type,
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
        }
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl MulAssign for PointOp {
    fn mul_assign(&mut self, other: PointOp) {
        let names = union_names(self.names.clone(), &other.names);
        *self = PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l: self.l * other.l,
            reverb: other.reverb,
            osc_type: other.osc_type,
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
        }
    }
}

impl PointOp {
    pub fn is_silent(&self) -> bool {
        let zero = Rational64::new(0, 1);
        self.fm == zero && self.fa < Rational64::new(20, 1) || self.g == zero
    }

    pub fn mod_by(&mut self, other: PointOp, l: Rational64) {
        let names = union_names(self.names.clone(), &other.names);
        *self = PointOp {
            fm: self.fm * other.fm,
            fa: self.fa + other.fa,
            pm: self.pm * other.pm,
            pa: self.pa + other.pa,
            g: self.g * other.g,
            l,
            reverb: other.reverb,
            osc_type: other.osc_type,
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
        }
    }

    pub fn init() -> PointOp {
        PointOp {
            fm: Ratio::new(1, 1),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(0, 1),
            g: Ratio::new(1, 1),
            l: Ratio::new(1, 1),
            reverb: Ratio::new(0, 1),
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            asr: ASR::Long,
            portamento: Ratio::new(1, 1),
            osc_type: OscType::Sine { pow: None },
            names: NameSet::new(),
        }
    }
    pub fn init_silent() -> PointOp {
        PointOp {
            fm: Ratio::new(0, 1),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(0, 1),
            g: Ratio::new(0, 1),
            l: Ratio::new(1, 1),
            reverb: Ratio::new(0, 1),
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            portamento: Ratio::new(1, 1),
            asr: ASR::Long,
            osc_type: OscType::Sine { pow: None },
            names: NameSet::new(),
        }
    }
}

impl NormalForm {
    pub fn init() -> NormalForm {
        NormalForm {
            operations: vec![vec![PointOp::init()]],
            length_ratio: Ratio::new(1, 1),
        }
    }

    pub fn init_empty() -> NormalForm {
        NormalForm {
            operations: vec![],
            length_ratio: Ratio::new(0, 1),
        }
    }

    pub fn partition(&self, name: String) -> (NormalForm, NormalForm) {
        let silence = PointOp::init_silent();
        let mut named = NormalForm::init_empty();
        let mut rest = NormalForm::init_empty();

        for seq in self.operations.iter() {
            let elem_with_name = seq.iter().find(|&p_op| p_op.names.contains(&name));

            let mut named_seq = vec![];
            let mut rest_seq = vec![];
            match elem_with_name {
                Some(_) => {
                    for p_op in seq {
                        let mut name_op: PointOp;
                        let mut rest_op: PointOp;
                        if p_op.names.contains(&name) {
                            name_op = p_op.clone();
                            rest_op = p_op.clone() * silence.clone();
                            rest_op.fa = Rational64::new(0, 1);
                            rest_op.pa = Rational64::new(0, 1);
                        } else {
                            name_op = p_op.clone() * silence.clone();
                            name_op.fa = Rational64::new(0, 1);
                            name_op.pa = Rational64::new(0, 1);
                            rest_op = p_op.clone();
                        }

                        named_seq.push(name_op);
                        rest_seq.push(rest_op);
                    }

                    named.operations.push(named_seq);
                    rest.operations.push(rest_seq);
                }
                None => {
                    rest_seq = seq.clone();
                    rest.operations.push(rest_seq);
                }
            }
        }

        named.length_ratio = self.length_ratio;
        rest.length_ratio = self.length_ratio;

        (named, rest)
    }
}

#[cfg(test)]
mod normalize_tests;
