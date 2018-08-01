pub fn freq_to_sones(frequency: f32) -> f32 {
    // http://www.ukintpress-conferences.com/conf/08txeu_conf/pdf/day_1/01-06-garcia.pdf
    if frequency < 20.0 {
        0.0
    } else {
        1.0 / 2.0_f32.powf(((20.0 * (frequency).log10()) - 40.0) / 10.0)
    }
}

pub fn loudness_normalization(frequency: f32) -> f32 {
    let mut normalization = freq_to_sones(frequency);
    if normalization.is_nan() || normalization.is_infinite() || normalization > 1.0 {
        normalization = 1.0;
    };
    normalization
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_loudness_normalization() {
        let expected = loudness_normalization(0.0);
        let result = 0.0;
        assert_eq!(expected, result);

        let expected = loudness_normalization(10.0);
        let result = 0.0;
        assert_eq!(expected, result);

        let expected = loudness_normalization(100.0);
        let result = 1.0;
        assert_eq!(expected, result);

        let expected = loudness_normalization(250.0);
        let result = 0.5759918;
        assert_eq!(expected, result);

        let expected = loudness_normalization(500.0);
        let result = 0.3794706;
        assert_eq!(expected, result);

        let expected = loudness_normalization(1000.0);
        let result = 0.25;
        assert_eq!(expected, result);

        let expected = loudness_normalization(1500.0);
        let result = 0.19584954;
        assert_eq!(expected, result);
    }
}
