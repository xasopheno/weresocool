use crate::{NameSet, OscType, Term, ASR};
use num_rational::{Ratio, Rational64};
use scop::Defs;
use std::{
    collections::HashSet,
    ops::{Mul, MulAssign},
};
use weresocool_error::Error;
use weresocool_filter::BiquadFilterDef;
mod get_length_ratio;
pub mod helpers;
mod normalize;
pub mod substitute;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
/// All operations in the language take a NormalForm as an import and
/// return a NormalForm.
pub struct NormalForm {
    pub operations: Vec<Vec<PointOp>>,
    pub length_ratio: Rational64,
}

#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct PointOp {
    /// Frequency Multiply
    pub fm: Rational64,
    /// Frequency Add
    pub fa: Rational64,
    /// Pan Multiply
    pub pm: Rational64,
    /// Pan Add
    pub pa: Rational64,
    /// Gain Multiply
    pub g: Rational64,
    /// Length Multiply
    pub l: Rational64,
    /// Attack Length
    pub attack: Rational64,
    /// Decay Length
    pub decay: Rational64,
    /// Attack/Sustain/Release Type
    pub asr: ASR,
    /// Portamento Length
    pub portamento: Rational64,
    /// Reverb Multiplier
    pub reverb: Option<Rational64>,
    /// Oscillator Type
    pub osc_type: OscType,
    /// Set of Names
    pub names: NameSet,
    /// Filters
    pub filters: Vec<BiquadFilterDef>,
    /// Should fade out to nothing
    pub is_out: bool,
}

impl Default for PointOp {
    fn default() -> Self {
        PointOp {
            fm: Ratio::new(1, 1),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(0, 1),
            g: Ratio::new(1, 1),
            l: Ratio::new(1, 1),
            reverb: None,
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            asr: ASR::Long,
            portamento: Ratio::new(1, 1),
            osc_type: OscType::None,
            names: NameSet::new(),
            filters: vec![],
            is_out: false,
        }
    }
}

pub trait Normalize<T> {
    fn apply_to_normal_form(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs<T>,
    ) -> Result<(), Error>;
}

pub trait GetLengthRatio<T> {
    fn get_length_ratio(
        &self,
        normal_form: &NormalForm,
        defs: &mut Defs<T>,
    ) -> Result<Rational64, Error>;
}

pub trait Substitute<T> {
    fn substitute(&self, normal_form: &mut NormalForm, defs: &mut Defs<T>) -> Result<Term, Error>;
}

impl GetLengthRatio<Term> for NormalForm {
    fn get_length_ratio(
        &self,
        _normal_form: &NormalForm,
        _defs: &mut Defs<Term>,
    ) -> Result<Rational64, Error> {
        Ok(self.length_ratio)
    }
}

impl Substitute<Term> for NormalForm {
    fn substitute(
        &self,
        _normal_form: &mut NormalForm,
        _defs: &mut Defs<Term>,
    ) -> Result<Term, Error> {
        Ok(Term::Nf(self.clone()))
    }
}

pub fn union_names(b_tree_set: NameSet, left: &NameSet) -> NameSet {
    let mut result = b_tree_set;
    for val in left.to_vec() {
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

impl Normalize<Term> for NormalForm {
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        _defs: &mut Defs<Term>,
    ) -> Result<(), Error> {
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
            reverb: if other.reverb.is_none() {
                self.reverb
            } else {
                other.reverb
            },
            osc_type: if other.osc_type.is_none() {
                self.osc_type
            } else {
                other.osc_type
            },
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
            filters: self
                .filters
                .iter()
                .chain(&other.filters)
                .map(|f| f.to_owned())
                .collect(),
            is_out: other.is_out,
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
            reverb: if other.reverb.is_none() {
                self.reverb
            } else {
                other.reverb
            },
            osc_type: if other.osc_type.is_none() {
                self.osc_type.clone()
            } else {
                other.osc_type.clone()
            },
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
            filters: self
                .filters
                .iter()
                .chain(&other.filters)
                .map(|f| f.to_owned())
                .collect(),
            is_out: other.is_out,
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
            reverb: if other.reverb.is_none() {
                self.reverb
            } else {
                other.reverb
            },
            osc_type: if other.osc_type == OscType::None {
                self.osc_type.clone()
            } else {
                other.osc_type
            },
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
            filters: self
                .filters
                .iter()
                .chain(&other.filters)
                .map(|f| f.to_owned())
                .collect(),
            is_out: other.is_out,
        }
    }
}

impl PointOp {
    pub fn is_silent(&self) -> bool {
        let zero = Rational64::new(0, 1);
        self.fm == zero && self.fa < Rational64::new(20, 1) || self.g == zero
    }

    pub fn silence(&mut self) {
        self.fm = Rational64::from_integer(0);
        self.fa = Rational64::from_integer(0);
        self.g = Rational64::from_integer(0);
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
            reverb: if other.reverb.is_none() {
                self.reverb
            } else {
                other.reverb
            },
            osc_type: if other.osc_type.is_none() {
                self.osc_type.clone()
            } else {
                other.osc_type
            },
            attack: self.attack * other.attack,
            decay: self.decay * other.decay,
            asr: other.asr,
            portamento: self.portamento * other.portamento,
            names,
            filters: self
                .filters
                .iter()
                .chain(&other.filters)
                .map(|f| f.to_owned())
                .collect(),
            is_out: other.is_out,
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
            reverb: None,
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            asr: ASR::Long,
            portamento: Ratio::new(1, 1),
            osc_type: OscType::None,
            names: NameSet::new(),
            filters: vec![],
            is_out: false,
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
            reverb: None,
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            portamento: Ratio::new(1, 1),
            asr: ASR::Long,
            osc_type: OscType::None,
            names: NameSet::new(),
            filters: vec![],
            is_out: false,
        }
    }

    //        pub fn to_op(&self) -> Op {
    //            let osc_op = match self.osc_type {
    //                OscType::Sine => Op::Sine,
    //                OscType::Square => Op::Square,
    //                OscType::Noise => Op::Noise,
    //            };
    //            Op::Compose {
    //                operations: vec![
    //                    osc_op,
    //                    Op::TransposeM { m: self.fm },
    //                    Op::TransposeA { a: self.fa },
    //                    Op::PanM { m: self.pm },
    //                    Op::PanA { a: self.pa },
    //                    Op::Gain { m: self.g },
    //                    Op::Length { m: self.l },
    //                ],
    //            }
    //        }
}

impl NormalForm {
    /// Creates a NormalForm with a single PointOp in the operations
    /// and sets the appropriate length_ratio.
    pub fn init() -> NormalForm {
        NormalForm {
            operations: vec![vec![PointOp::init()]],
            length_ratio: Ratio::new(1, 1),
        }
    }

    /// Creates a NormalForm with empty operations
    /// and set the length_ratio to zero.
    pub fn init_empty() -> NormalForm {
        NormalForm {
            operations: vec![],
            length_ratio: Ratio::new(0, 1),
        }
    }

    /// Applys function 'f' to every PointOp in the NormalForm
    pub fn fmap_mut(&mut self, f: impl Fn(&mut PointOp)) {
        for voice in self.operations.iter_mut() {
            for point_op in voice {
                f(point_op)
            }
        }
    }

    /// Applys function 'f' to every PointOp in the NormalForm
    /// with an impl FnMut allows a mutable function to be passed in.
    pub fn fmap_with_state(&self, mut f: impl FnMut(&PointOp)) {
        for voice in self.operations.iter() {
            for point_op in voice {
                f(point_op)
            }
        }
    }

    /// Given a name, solos that name by calling op.silence() on every op
    /// that doesn't have that name in their NameSet.
    pub fn solo_ops_by_name(&mut self, name: &str) {
        self.fmap_mut(|op: &mut PointOp| {
            if !op.names.contains(name) {
                op.silence();
            };
        })
    }

    /// Returns all the names that exist in the NormalForm
    pub fn names(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        self.fmap_with_state(|op| {
            for name in op.names.to_vec() {
                result.insert(name.clone());
            }
        });

        result
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
