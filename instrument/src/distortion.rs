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
