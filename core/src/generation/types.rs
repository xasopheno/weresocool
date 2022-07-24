use num_rational::Rational64;
use scop::Defs;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::path::PathBuf;
use weresocool_ast::{NameSet, NormalForm, Normalize, OscType, PointOp, Term, ASR};
use weresocool_error::Error;
use weresocool_instrument::Basis;

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
