use crate::generation::Op4D;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use weresocool_ast::{NameSet, OscType, PointOp, ASR};
use weresocool_instrument::Basis;
use weresocool_shared::r_to_f64;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EventType {
    On,
    Off,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimedOp {
    pub t: Rational64,
    pub event_type: EventType,
    pub voice: usize,
    pub event: usize,
    pub attack: Rational64,
    pub decay: Rational64,
    pub reverb: Rational64,
    pub asr: ASR,
    pub portamento: Rational64,
    pub osc_type: OscType,
    pub fm: Rational64,
    pub fa: Rational64,
    pub pm: Rational64,
    pub pa: Rational64,
    pub g: Rational64,
    pub l: Rational64,
    pub names: Vec<String>,
}

impl TimedOp {
    pub fn to_op_4d(&self, basis: &Basis) -> Op4D {
        let zero = Rational64::new(0, 1);
        let is_silent = (self.fm == zero && self.fa < Rational64::new(20, 1)) || self.g == zero;
        let y = if is_silent {
            0.0
        } else {
            r_to_f64(basis.f).mul_add(r_to_f64(self.fm), r_to_f64(self.fa))
        };
        let z = if is_silent {
            0.0
        } else {
            r_to_f64(basis.g) * r_to_f64(self.g)
        };
        Op4D {
            l: r_to_f64(self.l) * r_to_f64(basis.l),
            t: r_to_f64(self.t) * r_to_f64(basis.l),
            x: ((r_to_f64(basis.p) + r_to_f64(self.pa)) * r_to_f64(self.pm)),
            y: y.log10(),
            z,
            voice: self.voice,
            event: self.event,
            names: self.names.to_owned(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn to_point_op(&self) -> PointOp {
        PointOp {
            fm: self.fm,
            fa: self.fa,
            pm: self.pm,
            pa: self.pa,
            g: self.g,
            l: self.l,
            reverb: Some(self.reverb),
            attack: self.decay,
            decay: self.decay,
            asr: self.asr,
            portamento: self.portamento,
            osc_type: self.osc_type,
            names: NameSet::new(),
            filters: Vec::new(),
        }
    }

    pub fn from_point_op(
        point_op: &PointOp,
        time: &mut Rational64,
        voice: usize,
        event: usize,
    ) -> Self {
        let timed_op = Self {
            fm: point_op.fm,
            fa: point_op.fa,
            pm: point_op.pm,
            pa: point_op.pa,
            attack: point_op.attack,
            osc_type: point_op.osc_type,
            decay: point_op.decay,
            reverb: point_op
                .reverb
                .unwrap_or_else(|| Rational64::from_integer(0)),
            asr: point_op.asr,
            portamento: point_op.portamento,
            g: point_op.g,
            l: point_op.l,
            t: *time,
            event_type: EventType::On,
            voice,
            event,
            names: point_op.names.to_vec(),
        };

        *time += point_op.l;

        timed_op
    }
}
