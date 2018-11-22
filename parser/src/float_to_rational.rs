
pub mod helpers {
    extern crate num_rational;
    extern crate regex;
    use regex::Regex;
    use num_rational::{Ratio, Rational};
    use std::str::FromStr;

    pub fn f32_to_rational(float_string: String) -> Rational {
        let mut s =
            float_string
            .to_string();
//            .chars();
        let re = Regex::new(r"\.(.*)$").unwrap();
        let decimal = re.captures(&s).unwrap()[1].to_string().clone();
        let d = decimal.len();
        let num = f32::from_str().unwrap() * d;

       Ratio::new(float, 1)
    }

    #[cfg(test)]
    pub mod tests {
        use super::*;


     #[test]
        fn test_float_to_rational() {
            let result = f32_to_rational("112.999".to_string());
            let expected = Ratio::new(10001, 10000);
            assert_eq!(result, expected);
        }
    }
}
