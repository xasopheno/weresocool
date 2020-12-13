use num::{
    bigint::{BigInt, Sign},
    ToPrimitive,
};
use num_rational::BigRational;
use std::str::FromStr;

pub type CoolRatio = BigRational;

pub trait CoolRatioT {
    fn from_int(numer: i64) -> Self;
    fn from_ints(numer: i64, denom: i64) -> Self;
    fn as_f64(&self) -> f64;
    fn is_zero(&self) -> bool;
    fn from_float_string(float_string: String) -> Self;
}

impl CoolRatioT for CoolRatio {
    fn from_float_string(float_string: String) -> Self {
        let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
        let den = i64::pow(10, decimal.len() as u32);
        let num = i64::from_str(&float_string.replace('.', "")).unwrap();

        Self::from_ints(num, den)
    }

    fn from_int(numer: i64) -> Self {
        let numer_sign = if numer.signum() == -1 {
            Sign::Minus
        } else {
            Sign::Plus
        };

        let numer = numer.abs() as u32;

        BigRational::new(
            BigInt::new(numer_sign, vec![numer]),
            BigInt::new(Sign::Plus, vec![1]),
        )
    }

    fn from_ints(numer: i64, denom: i64) -> Self {
        let numer_sign = if numer.signum() == denom.signum() {
            Sign::Plus
        } else {
            Sign::Minus
        };

        let numer = numer.abs() as u32;
        let denom = denom.abs() as u32;

        BigRational::new(
            BigInt::new(numer_sign, vec![numer]),
            BigInt::new(Sign::Plus, vec![denom]),
        )
    }

    fn as_f64(&self) -> f64 {
        self.to_f64()
            .unwrap_or_else(|| panic!("CoolRatio::as_f64 failed with value {:?}", self))
    }

    fn is_zero(&self) -> bool {
        *self == BigRational::from_int(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cool_ratio_test() {
        let a = BigRational::from_ints(5, 4);
        let b = BigRational::from_ints(-1, 4);
        let c = BigRational::from_int(1);

        assert_eq!(b.as_f64(), -0.25);
        assert_eq!(&a + &b, c);
        assert_eq!((&c - &c).is_zero(), true);
    }
}
