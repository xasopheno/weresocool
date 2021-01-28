use float_cmp::ApproxEq;
use num_rational::{Ratio, Rational64};
use std::str::FromStr;

pub fn et_to_rational(i: i64, d: usize) -> Rational64 {
    let signum = i.signum();
    if signum == 0 {
        return Rational64::from_integer(1);
    }

    let et = 2.0_f32.powf(i as f32 / d as f32);
    if signum == -1 {
        let result = f32_string_to_rational(format!("{:.8}", et));
        result.recip();
        result
    } else {
        f32_string_to_rational(format!("{:.8}", et))
    }
}

pub fn lossy_rational_mul(a: Rational64, b: Rational64) -> Rational64 {
    f32_string_to_rational(format!("{:.8}", r_to_f64(a) * r_to_f64(b)))
}

pub fn f32_string_to_rational(float_string: String) -> Rational64 {
    let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
    let den = i64::pow(10, decimal.len() as u32);
    let num = i64::from_str(&float_string.replace('.', "")).unwrap();

    Ratio::new(num, den)
}

pub fn f32_to_rational(mut float: f32) -> Rational64 {
    if !float.is_finite() || float > 1_000_000.0 {
        float = 0.0
    }
    let float_string = format!("{:.8}", float);
    let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
    let den = i64::pow(10, decimal.len() as u32);
    let num = i64::from_str(&float_string.replace('.', "")).unwrap();

    Ratio::new(num, den)
}

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

pub fn cmp_vec_f32(vec1: Vec<f32>, vec2: Vec<f32>) -> bool {
    for (a, b) in vec1.iter().zip(vec2) {
        if !a.approx_eq(b, (0.0, 2)) {
            return false;
        }
    }
    true
}

pub fn cmp_vec_f64(vec1: Vec<f64>, vec2: Vec<f64>) -> bool {
    for (a, b) in vec1.iter().zip(vec2) {
        if !a.approx_eq(b, (0.0, 2)) {
            return false;
        }
    }
    true
}

pub fn cmp_f32(a: f32, b: f32) -> bool {
    a.approx_eq(b, (0.0, 2))
}

pub fn cmp_f64(a: f64, b: f64) -> bool {
    a.approx_eq(b, (0.0, 2))
}
