use num::{
    bigint::{BigInt, Sign},
    ToPrimitive,
};
use num_rational::BigRational;

pub type CoolRatio = BigRational;

pub trait CoolRatioT {
    fn from_int(numer: i32) -> Self;
    fn from_ints(numer: i32, denom: i32) -> Self;
    fn as_f64(&self) -> f64;
    fn is_zero(&self) -> bool;
}

impl CoolRatioT for CoolRatio {
    fn from_int(numer: i32) -> Self {
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

    fn from_ints(numer: i32, denom: i32) -> Self {
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
        self.to_f64().unwrap()
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
        assert_eq!(a + b, c);
    }
}
