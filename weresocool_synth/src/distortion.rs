#![allow(unused)]

use serde::{Deserialize, Serialize};

/// Runtime distortion effect definition (f64 params for synthesis)
/// Copy-able for efficiency - small, cache-friendly
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DistortionDef {
    Wavefolder { threshold: f64, stages: u8, input_gain: f64, output_gain: f64 },
    SoftClip { threshold: f64, input_gain: f64, output_gain: f64 },
    Overdrive { input_gain: f64, output_gain: f64 },
    Bitcrusher { bits: u8, input_gain: f64, output_gain: f64 },
    Tanh { input_gain: f64, output_gain: f64 },
}

/// Wavefold a sample. Output stays within ±threshold (dynamics preserved).
#[inline(always)]
pub fn wavefold(sample: f64, threshold: f64, drive: f64, stages: u8) -> f64 {
    let threshold = threshold.abs().max(0.001);
    let mut x = sample * drive;

    for _ in 0..stages {
        while x > threshold {
            x = 2.0 * threshold - x;
        }
        while x < -threshold {
            x = -2.0 * threshold - x;
        }
    }

    x // Dynamics preserved
}

/// Process sample through a single distortion effect
#[inline(always)]
pub fn process_distortion(sample: f64, distortion: &DistortionDef) -> f64 {
    match distortion {
        DistortionDef::Wavefolder { threshold, stages, input_gain, output_gain } => {
            wavefold(sample, *threshold, *input_gain, *stages) * output_gain
        }
        DistortionDef::SoftClip { threshold, input_gain, output_gain } => {
            soft_clip(sample, *input_gain, *threshold) * output_gain
        }
        DistortionDef::Overdrive { input_gain, output_gain } => {
            overdrive(sample, *input_gain) * output_gain
        }
        DistortionDef::Bitcrusher { bits, input_gain, output_gain } => {
            bitcrusher(sample * input_gain, *bits) * output_gain
        }
        DistortionDef::Tanh { input_gain, output_gain } => {
            (sample * input_gain).tanh() * output_gain
        }
    }
}

/// Process sample through entire distortion chain (no allocations)
#[inline(always)]
pub fn process_distortions(mut sample: f64, distortions: &[DistortionDef]) -> f64 {
    for d in distortions {
        sample = process_distortion(sample, d);
    }
    sample
}

pub fn distort_hard_clipping(sample: f64, gain: f64, threshold: f64) -> f64 {
    let amplified = sample * gain;

    if amplified > threshold {
        threshold
    } else if amplified < -threshold {
        -threshold
    } else {
        amplified
    }
}

pub fn soft_clip(sample: f64, gain: f64, threshold: f64) -> f64 {
    let amplified = sample * gain;

    if amplified > threshold {
        threshold
            + (1.0 - threshold) * (amplified - threshold)
                / (1.0 + ((amplified - threshold) / (1.0 - threshold)).powi(2))
    } else if amplified < -threshold {
        -threshold
            + (1.0 - threshold) * (amplified + threshold)
                / (1.0 + ((amplified + threshold) / (1.0 - threshold)).powi(2))
    } else {
        amplified
    }
}

pub fn wave_shaping(sample: f64, gain: f64) -> f64 {
    (3.0 + gain) * sample.powi(2) - 2.0 * sample.powi(3)
}

pub fn overdrive(sample: f64, gain: f64) -> f64 {
    let x = sample * gain;
    if x < -3.0 {
        -1.0
    } else if x > 3.0 {
        1.0
    } else {
        x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
    }
}

// pub fn arctan_distortion(sample: f64, gain: f64) -> f64 {
// (sample * gain).atan() / std::f64::consts::PI_2
// }

pub fn fuzz_distortion(sample: f64, gain: f64) -> f64 {
    let x = sample * gain;
    x / (1.0 - x.abs())
}

pub fn power_distortion(sample: f64, exponent: f64) -> f64 {
    sample.abs().powf(exponent) * sample.signum()
}

pub fn exponential_distortion(sample: f64, gain: f64) -> f64 {
    if sample >= 0.0 {
        (1.0 - (-sample * gain).exp()).max(1.0)
    } else {
        -((1.0 - (sample * gain).exp()).max(1.0))
    }
}

pub fn bitcrusher(sample: f64, bit_depth: u8) -> f64 {
    let scale = (2.0_f64.powi(bit_depth as i32) / 2.0) - 1.0;
    (sample * scale).round() / scale
}

pub fn cubic_distortion(sample: f64, gain: f64) -> f64 {
    let x = sample * gain;
    x * x * x
}
