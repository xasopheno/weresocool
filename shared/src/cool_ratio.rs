use num::bigint::{BigInt, Sign};
use num_rational::BigRational;

pub trait CoolRatio {
    fn from_int(n: i32, d: i32) -> Self;
}

impl CoolRatio for BigRational {
    fn from_int(n: i32, d: i32) -> Self {
        let n_sign = if n.signum() == d.signum() {
            Sign::Plus
        } else {
            Sign::Minus
        };

        let n = u32::from_str_radix(&n.abs().to_string(), 10).unwrap();
        let d = u32::from_str_radix(&d.abs().to_string(), 10).unwrap();

        BigRational::new(
            BigInt::new(n_sign, vec![n]),
            BigInt::new(Sign::Plus, vec![d]),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cool_ratio_test() {
        let a = BigRational::from_int(2, 1);
        let b = BigRational::from_int(-3, 1);
        let c = BigRational::from_int(-1, 1);

        assert_eq!(a + b, c)
    }
}
