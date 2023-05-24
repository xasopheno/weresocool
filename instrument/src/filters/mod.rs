use std::f64::consts::PI;
use weresocool_shared::Settings;

const TAU: f64 = PI * 2.0;

#[derive(Clone, Debug, PartialEq)]
pub struct BiquadFilter {
    // Feedforward coefficients for the current and two previous inputs
    feedforward_coefs: [f64; 3],
    // Feedback coefficients for the two previous outputs
    feedback_coefs: [f64; 3],
    // The two most recent input values
    input_history: [f64; 2],
    // The two most recent output values
    output_history: [f64; 2],
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

    // The process function filters a sample and returns the filtered value
    pub fn process(&mut self, input_sample: f64) -> f64 {
        // Filter equation
        let filtered_output = self.feedforward_coefs[0] * input_sample
            + self.feedforward_coefs[1] * self.input_history[0]
            + self.feedforward_coefs[2] * self.input_history[1]
            - self.feedback_coefs[1] * self.output_history[0]
            - self.feedback_coefs[2] * self.output_history[1];

        // Update the history of inputs and outputs
        self.input_history[1] = self.input_history[0];
        self.input_history[0] = input_sample;
        self.output_history[1] = self.output_history[0];
        self.output_history[0] = filtered_output;

        filtered_output
    }
}

// The lowpass function generates the feedforward and feedback coefficients needed for a lowpass filter
pub fn lowpass(cutoff_frequency: f64, quality_factor: f64) -> ([f64; 3], [f64; 3]) {
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

pub fn highpass(cutoff_frequency: f64, quality_factor: f64) -> ([f64; 3], [f64; 3]) {
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

pub fn bandpass(center_frequency: f64, quality_factor: f64) -> ([f64; 3], [f64; 3]) {
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
