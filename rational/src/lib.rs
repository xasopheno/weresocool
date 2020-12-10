use num_bigint::{BigInt, Sign};
use num_rational::BigRational;

pub type BigRatio = BigRational;

pub trait Rational {
    fn new(n: u32, d: u32) -> Self;
}

impl Rational for BigRatio {
    fn new(n: u32, d: u32) -> Self {
        Self::new(
            BigInt::new(Sign::Plus, vec![n as u32]),
            BigInt::new(Sign::Plus, vec![d as u32]),
        )
    }
}
