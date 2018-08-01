macro_rules! r {
    ($(($num:expr,$den:expr,$offset:expr,$gain:expr,$pan:expr)),*$(,)*) => {
        vec![$(
            R::atio($num,$den,$offset, $gain * ((-1.0 + $pan) / -2.0), Pan::Left),
            R::atio($num,$den,$offset, $gain * ((1.0 + $pan) / 2.0), Pan::Right),
        )*]
    }
}

#[cfg(test)]
pub mod tests {
    use ratios::{R, Pan};

    #[test]
    fn test_r_macro() {
        let r_macro = r![
            (3, 2, 0.0, 0.6, -1.0),
            (5, 4, 1.5, 0.5, 0.5)
        ];

        let result = vec![
            R::atio(3, 2, 0.0, 0.6, Pan::Left),
            R::atio(3, 2, 0.0, 0.0, Pan::Right),
            R::atio(5, 4, 1.5, 0.125, Pan::Left),
            R::atio(5, 4, 1.5, 0.375, Pan::Right),
        ];

        assert_eq!(r_macro, result);
    }
}