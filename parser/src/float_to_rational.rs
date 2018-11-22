
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
        let decimal = s.split(".").collect::<Vec<&str>>()[1];
        println!("{:?}", decimal);
        let denometer = isize::pow(10, decimal.len() as u32);
        println!("{:?}", denometer);
        let num = isize::from_str(&s.replace(".", "")).unwrap();

       Ratio::new(num, denometer)
    }

    #[cfg(test)]
    pub mod tests {
        use super::*;


     #[test]
        fn test_float_to_rational() {
            let result = f32_to_rational("112.999".to_string());
            let expected = Ratio::new(112999, 1000);
            assert_eq!(result, expected);
        }
    }
}
