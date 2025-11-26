use crate::{color::ColorMap, NameSet, OscType, Term, ASR, Distortion, wgsl::WgslMap, rand_ctx::RandCtx};
use num_rational::{Ratio, Rational64};
use scop::Defs as ScopDefs;
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

#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct ColorGrading {
    pub hue: Rational64,
    pub saturation: Rational64,
    pub brightness: Rational64,
    pub vibrance: Rational64,
    pub gamma: Rational64,
}

impl Default for ColorGrading {
    fn default() -> Self {
        ColorGrading {
            hue: Ratio::new(0, 1),
            saturation: Ratio::new(1, 1),
            brightness: Ratio::new(0, 1),
            vibrance: Ratio::new(0, 1),
            gamma: Ratio::new(1, 1),
        }
    }
}

impl ColorGrading {
    /// Check if this ColorGrading is at identity/default values (no transformation)
    pub fn is_identity(&self) -> bool {
        let zero = Ratio::new(0, 1);
        let one = Ratio::new(1, 1);

        self.hue == zero
            && self.saturation == one
            && self.brightness == zero
            && self.vibrance == zero
            && self.gamma == one
    }
}

impl Mul<ColorGrading> for ColorGrading {
    type Output = ColorGrading;

    fn mul(self, other: ColorGrading) -> ColorGrading {
        ColorGrading {
            hue: self.hue + other.hue,
            saturation: self.saturation * other.saturation,
            brightness: self.brightness + other.brightness,
            vibrance: self.vibrance + other.vibrance,
            gamma: self.gamma * other.gamma,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Defs {
    pub ops: ScopDefs<Term>,
    pub colors: ColorMap,
    pub wgsl: WgslMap,
    pub rand_ctx: RandCtx,
}

impl Default for Defs {
    fn default() -> Self {
        let random_seed: u128 = {
            let mut buf = [0u8; 16];
            getrandom::getrandom(&mut buf).expect("Failed to get random bytes");
            u128::from_le_bytes(buf)
        };

        Defs {
            ops: ScopDefs::new(),
            colors: ColorMap::new(),
            wgsl: WgslMap::new(),
            rand_ctx: RandCtx::from_u128(random_seed),
        }
    }
}

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
    /// Distortion effects (wavefolder, etc.)
    pub distortions: Vec<Distortion>,
    /// Should fade out to nothing
    pub is_out: bool,
    pub follows: Vec<crate::follow::types::FollowNF>,
    pub colors: Vec<u64>,
    /// WGSL block IDs
    pub wgsl: Vec<u64>,
    /// MIDI targets (channels)
    pub midi: Vec<u8>,
    /// Color grading adjustments
    pub color_grading: ColorGrading,
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
            distortions: vec![],
            is_out: false,
            follows: vec![],
            colors: vec![],
            wgsl: vec![],
            midi: vec![],
            color_grading: ColorGrading::default(),
        }
    }
}

pub trait Normalize {
    fn apply_to_normal_form(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<(), Error>;
}

pub trait GetLengthRatio {
    fn get_length_ratio(
        &self,
        normal_form: &NormalForm,
        defs: &mut Defs,
    ) -> Result<Rational64, Error>;
}

pub trait Substitute {
    fn substitute(&self, normal_form: &mut NormalForm, defs: &mut Defs) -> Result<Term, Error>;
}

impl GetLengthRatio for NormalForm {
    fn get_length_ratio(
        &self,
        _normal_form: &NormalForm,
        _defs: &mut Defs,
    ) -> Result<Rational64, Error> {
        Ok(self.length_ratio)
    }
}

impl Substitute for NormalForm {
    fn substitute(
        &self,
        _normal_form: &mut NormalForm,
        _defs: &mut Defs,
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

impl Normalize for NormalForm {
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        _defs: &mut Defs,
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
            distortions: self
                .distortions
                .iter()
                .chain(&other.distortions)
                .copied()
                .collect(),
            is_out: other.is_out,
            follows: self
                .follows
                .iter()
                .cloned()
                .chain(other.follows.iter().cloned())
                .collect(),
            colors: self.colors.iter().chain(&other.colors).map(|c| c.to_owned()).collect(),
            wgsl: self.wgsl.iter().chain(&other.wgsl).map(|c| c.to_owned()).collect(),
            midi: self.midi.iter().chain(&other.midi).cloned().collect(),
            color_grading: self.color_grading * other.color_grading,
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<'a> Mul<&'a PointOp> for &PointOp {
    type Output = PointOp;

    fn mul(self, other: &'a PointOp) -> PointOp {
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
            distortions: self
                .distortions
                .iter()
                .chain(&other.distortions)
                .copied()
                .collect(),
            is_out: other.is_out,
            follows: self
                .follows
                .iter()
                .cloned()
                .chain(other.follows.iter().cloned())
                .collect(),
            colors: self
                .colors
                .iter()
                .chain(&other.colors)
                .map(|c| c.to_owned())
                .collect(),
            wgsl: self
                .wgsl
                .iter()
                .chain(&other.wgsl)
                .map(|c| c.to_owned())
                .collect(),
            midi: self.midi.iter().chain(&other.midi).cloned().collect(),
            color_grading: self.color_grading.clone() * other.color_grading.clone(),
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
            distortions: self
                .distortions
                .iter()
                .chain(&other.distortions)
                .copied()
                .collect(),
            is_out: other.is_out,
            follows: self
                .follows
                .iter()
                .cloned()
                .chain(other.follows.iter().cloned())
                .collect(),
            colors: self
                .colors
                .iter()
                .chain(&other.colors)
                .map(|c| c.to_owned())
                .collect(),
            wgsl: self
                .wgsl
                .iter()
                .chain(&other.wgsl)
                .map(|c| c.to_owned())
                .collect(),
            midi: self.midi.iter().chain(&other.midi).cloned().collect(),
            color_grading: self.color_grading.clone() * other.color_grading,
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
            distortions: self
                .distortions
                .iter()
                .chain(&other.distortions)
                .copied()
                .collect(),
            is_out: other.is_out,
            follows: self
                .follows
                .iter()
                .cloned()
                .chain(other.follows.iter().cloned())
                .collect(),
            colors: self
                .colors
                .iter()
                .chain(&other.colors)
                .map(|c| c.to_owned())
                .collect(),
            wgsl: self
                .wgsl
                .iter()
                .chain(&other.wgsl)
                .map(|c| c.to_owned())
                .collect(),
            midi: self.midi.iter().chain(&other.midi).cloned().collect(),
            color_grading: self.color_grading.clone() * other.color_grading,
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
            wgsl: vec![],
            midi: vec![],
            ..Default::default()
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
            wgsl: vec![],
            midi: vec![],
            ..Default::default()
        }
    }

    /// Get transformed color IDs by applying color grading to each color
    /// and inserting the result back into the ColorMap.
    /// This ensures same color + same grading = same ID (deduplication).
    pub fn get_transformed_colors(&self, color_map: &mut ColorMap) -> Vec<u64> {
        use crate::color::{apply_color_grading, ColorValue};

        // Check if color grading is at default (identity) - if so, skip transformation
        if self.color_grading.is_identity() {
            return self.colors.clone();
        }

        self.colors
            .iter()
            .map(|color_id| {
                // Look up the original color
                let color_value = color_map.get_by_hash(color_id.to_string());

                if let Some(cv) = color_value {
                    // Extract a Color from the ColorValue
                    let color = cv.extract_color();

                    // Apply color grading transformations
                    let transformed = apply_color_grading(&color, &self.color_grading);

                    // Insert back into ColorMap as a ColorSet with single element
                    // This will reuse existing ID if same transformed color exists
                    color_map.insert(ColorValue::ColorSet {
                        colors: vec![transformed],
                    })
                } else {
                    // If color not found, return original ID
                    *color_id
                }
            })
            .collect()
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
