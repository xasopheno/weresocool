use crate::generation::Op4D;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use weresocool_ast::{Ext, NameSet, OscType, PointOp, ASR};
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
    /// The extension registry, carried WHOLESALE from the PointOp — one field
    /// list exists (Ext's own), so nothing is silently dropped on the way to
    /// Op4D/JSON/CSV. (Historically colors+wgsl were hand-copied here and
    /// fade/layer/midi/fit/grading were lost.)
    pub ext: Ext,
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
            colors: self.ext.visual.colors.iter().map(|c| c.to_string()).collect(),
            wgsl: self.ext.visual.wgsl.clone(),
            // Carried from the ext registry (historically hardcoded to
            // None/1.0 here — gradient data silently never reached JSON).
            color_gradient: self.ext.visual.color_distribution.gradient,
            color_mix: self.ext.visual.color_distribution.mix,
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
            // NOTE (pre-existing): `attack` is filled from `decay` because
            // `TimedOp` has never carried an attack of its own.
            attack: self.decay,
            decay: self.decay,
            // `TimedOp` is the JSON/CSV/FromSound shape, and it does not carry
            // the envelope. Identity here, deliberately: a round-trip through
            // this type returns an UNARTICULATED note rather than a randomly
            // shaped one. If `Env` should survive `FromSound`, these three
            // have to be added to `TimedOp` itself — which changes the
            // exported JSON schema, so it is not a silent decision.
            sustain: Rational64::new(1, 1),
            release: Rational64::new(1, 1),
            gate: Rational64::new(1, 1),
            asr: self.asr,
            // TimedOp is the JSON/CSV shape and carries no subdivision info.
            continues: false,
            portamento: self.portamento,
            osc_type: self.osc_type.clone(),
            names: NameSet::new(),
            filters: Vec::new(),
            distortions: Vec::new(),
            //TODO
            is_out: false,
            follows: vec![],
            ext: self.ext.clone(),
            phase: None,
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
            osc_type: point_op.osc_type.clone(),
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
            ext: point_op.ext.clone(),
        };

        *time += point_op.l;

        timed_op
    }
}
