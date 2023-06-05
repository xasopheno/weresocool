pub mod asdr;
use asdr::AdsrEnvelope;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use weresocool_shared::{r_to_f64, Settings};

const TAU: f64 = PI * 2.0;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize)]
pub enum BiquadFilterType {
    Lowpass,
    Highpass,
    Bandpass,
    // Notch,
    // Peak,
    // LowShelf,
    // HighShelf,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize)]

pub struct ASDRFilterDef {
    pub a: Rational64,
    pub s: Rational64,
    pub d: Rational64,
    pub r: Rational64,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize)]

pub struct BiquadFilterDef {
    pub filter_type: BiquadFilterType,
    pub cutoff_frequency: Rational64,
    pub q_factor: Rational64,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize)]
pub enum FilterDef {
    Biquad { hash: String, def: BiquadFilterDef },
    ASDR { hash: String, def: ASDRFilterDef },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Filter {
    Biquad { hash: String, def: BiquadFilter },
    ASDR { hash: String, def: AdsrEnvelope },
}

impl Filter {
    pub fn hash(&self) -> &String {
        match self {
            Filter::Biquad { hash, .. } => hash,
            Filter::ASDR { hash, .. } => hash,
        }
    }
    pub fn process(&mut self, sample: f64) -> f64 {
        match self {
            Filter::Biquad { def, .. } => def.process(sample),
            Filter::ASDR { def, .. } => def.process(sample),
        }
    }
}

impl FilterDef {
    pub fn hash(&self) -> &String {
        match self {
            FilterDef::Biquad { hash, .. } => hash,
            FilterDef::ASDR { hash, .. } => hash,
        }
    }

    pub fn to_filter(&self) -> Filter {
        match self {
            FilterDef::Biquad { def, hash } => match def.filter_type {
                BiquadFilterType::Lowpass => Filter::Biquad {
                    hash: hash.clone(),
                    def: lowpass_filter(r_to_f64(def.cutoff_frequency), r_to_f64(def.q_factor)),
                },
                BiquadFilterType::Highpass => Filter::Biquad {
                    hash: hash.clone(),
                    def: highpass_filter(r_to_f64(def.cutoff_frequency), r_to_f64(def.q_factor)),
                },
                BiquadFilterType::Bandpass => Filter::Biquad {
                    hash: hash.clone(),
                    def: bandpass_filter(r_to_f64(def.cutoff_frequency), r_to_f64(def.q_factor)),
                },
                // BiquadFilterType::Highpass => highpass_filter(
                // biquad.hash.clone(),
                // r_to_f64(biquad.cutoff_frequency),
                // r_to_f64(biquad.q_factor),
                // ),
                // BiquadFilterType::Bandpass => highpass_filter(
                // biquad.hash.clone(),
                // r_to_f64(biquad.cutoff_frequency),
                // r_to_f64(biquad.q_factor),
                // ),
            },
            // Filter::ASDR(asdr) => asdr.to_filter(),
            _ => {
                panic!()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct BiquadFilter {
    feedforward_coefs: [f64; 3],
    // Feedback coefficients for the two previous outputs
    feedback_coefs: [f64; 3],
    // The two most recent input values
    input_history: [f64; 2],
    // The two most recent output values
    output_history: [f64; 2],
}

impl Ord for BiquadFilter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.feedforward_coefs[0]
            .partial_cmp(&other.feedforward_coefs[0])
            .unwrap_or(Ordering::Equal)
    }
}

impl Eq for BiquadFilter {}

impl Hash for BiquadFilter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for coef in &self.feedforward_coefs {
            coef.to_bits().hash(state);
        }
        for coef in &self.feedback_coefs {
            coef.to_bits().hash(state);
        }
        for hist in &self.input_history {
            hist.to_bits().hash(state);
        }
        for hist in &self.output_history {
            hist.to_bits().hash(state);
        }
    }
}

impl BiquadFilter {
    // The new function creates a BiquadFilter with given feedforward and feedback coefficients
    pub fn new(feedforward_coefs: [f64; 3], feedback_coefs: [f64; 3]) -> Self {
        BiquadFilter {
            feedforward_coefs,
            feedback_coefs,
            input_history: [0.0; 2],
            output_history: [0.0; 2],
        }
    }

    pub fn process(&mut self, input_sample: f64) -> f64 {
        let filtered_output = self.feedforward_coefs[0] * input_sample
            + self.feedforward_coefs[1] * self.input_history[0]
            + self.feedforward_coefs[2] * self.input_history[1]
            - self.feedback_coefs[1] * self.output_history[0]
            - self.feedback_coefs[2] * self.output_history[1];

        self.update_history(input_sample, filtered_output);
        filtered_output
    }

    fn update_history(&mut self, input_sample: f64, filtered_output: f64) {
        self.input_history.rotate_right(1);
        self.input_history[0] = input_sample;
        self.output_history.rotate_right(1);
        self.output_history[0] = filtered_output;
    }
}

pub fn lowpass_filter(cutoff_frequency: f64, quality_factor: f64) -> BiquadFilter {
    let (feedforward_coefs, feedback_coefs) = lowpass_coefs(cutoff_frequency, quality_factor);
    BiquadFilter::new(feedforward_coefs, feedback_coefs)
}

pub fn highpass_filter(cutoff_frequency: f64, quality_factor: f64) -> BiquadFilter {
    let (feedforward_coefs, feedback_coefs) = highpass_coefs(cutoff_frequency, quality_factor);
    BiquadFilter::new(feedforward_coefs, feedback_coefs)
}

pub fn bandpass_filter(cutoff_frequency: f64, quality_factor: f64) -> BiquadFilter {
    let (feedforward_coefs, feedback_coefs) = bandpass_coefs(cutoff_frequency, quality_factor);
    BiquadFilter::new(feedforward_coefs, feedback_coefs)
}

pub fn lowpass_coefs(cutoff_frequency: f64, quality_factor: f64) -> ([f64; 3], [f64; 3]) {
    // Calculate normalized cutoff frequency (w_c) and intermediate term alpha
    let normalized_cutoff = TAU * cutoff_frequency / Settings::global().sample_rate;
    let alpha = normalized_cutoff.sin() / (2.0 * quality_factor);
    let normalization_factor = 1.0 + alpha;

    // Calculate feedforward coefficients (b values)
    let feedforward_coefs = [
        (1.0 - normalized_cutoff.cos()) / 2.0 / normalization_factor,
        (1.0 - normalized_cutoff.cos()) / normalization_factor,
        (1.0 - normalized_cutoff.cos()) / 2.0 / normalization_factor,
    ];

    // Calculate feedback coefficients (a values)
    let feedback_coefs = [
        1.0,
        -2.0 * normalized_cutoff.cos() / normalization_factor,
        (1.0 - alpha) / normalization_factor,
    ];

    (feedforward_coefs, feedback_coefs)
}

pub fn highpass_coefs(cutoff_frequency: f64, quality_factor: f64) -> ([f64; 3], [f64; 3]) {
    // Calculate normalized cutoff frequency (w_c) and intermediate term alpha
    let normalized_cutoff = TAU * cutoff_frequency / Settings::global().sample_rate;
    let alpha = normalized_cutoff.sin() / (2.0 * quality_factor);
    let normalization_factor = 1.0 + alpha;

    // Calculate feedforward coefficients (b values)
    let feedforward_coefs = [
        (1.0 + normalized_cutoff.cos()) / 2.0 / normalization_factor,
        -(1.0 + normalized_cutoff.cos()) / normalization_factor,
        (1.0 + normalized_cutoff.cos()) / 2.0 / normalization_factor,
    ];

    // Calculate feedback coefficients (a values)
    let feedback_coefs = [
        1.0,
        -2.0 * normalized_cutoff.cos() / normalization_factor,
        (1.0 - alpha) / normalization_factor,
    ];

    (feedforward_coefs, feedback_coefs)
}

pub fn bandpass_coefs(center_frequency: f64, quality_factor: f64) -> ([f64; 3], [f64; 3]) {
    // Calculate normalized center frequency (w_0) and intermediate term alpha
    let normalized_center = TAU * center_frequency / Settings::global().sample_rate;
    let alpha = normalized_center.sin() / (2.0 * quality_factor);

    // Calculate feedforward coefficients (b values)
    let mut feedforward_coefs = [alpha, 0.0, -alpha];

    // Calculate feedback coefficients (a values)
    let mut feedback_coefs = [1.0 + alpha, -2.0 * normalized_center.cos(), 1.0 - alpha];

    // Normalize coefficients
    let normalization_factor = feedback_coefs[0];
    for coef in &mut feedforward_coefs {
        *coef /= normalization_factor;
    }
    for coef in &mut feedback_coefs {
        *coef /= normalization_factor;
    }

    (feedforward_coefs, feedback_coefs)
}
