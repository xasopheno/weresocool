#[cfg(test)]
mod tests {
    use crate::*;
    use weresocool_shared::Settings;

    #[test]
    fn test_biquad_filter_process() {
        let mut filter =
            BiquadFilter::new("test".to_string(), [-1.0, 1.0, -1.0], [-1.0, 1.0, -1.0]);
        let output = filter.process(1.0);

        assert_eq!(output, -1.0);
        assert_eq!(filter.input_history, [1.0, 0.0]);
        assert_eq!(filter.output_history, [-1.0, 0.0]);
    }

    #[test]
    fn test_biquad_filter_update_history() {
        Settings::init_test();
        let mut filter = BiquadFilter::new("test".to_string(), [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]);
        filter.update_history(1.0, 1.0);

        assert_eq!(filter.input_history, [1.0, 0.0]);
        assert_eq!(filter.output_history, [1.0, 0.0]);
    }

    #[test]
    fn test_biquad_filter_def_to_filter() {
        Settings::init_test();
        let def = BiquadFilterDef {
            hash: "test".to_string(),
            filter_type: BiquadFilterType::Lowpass,
            cutoff_frequency: Rational64::new(3, 2),
            q_factor: Rational64::new(1, 1),
        };
        let filter = def.to_filter();

        assert_eq!(filter.hash, "test".to_string());
    }

    #[test]
    fn test_biquad_filter_ord() {
        let filter_1 = BiquadFilter::new("test".to_string(), [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let filter_2 = BiquadFilter::new("test".to_string(), [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

        assert!(filter_1 < filter_2);
    }

    #[test]
    fn test_biquad_filter_hash() {
        let filter = BiquadFilter::new("test".to_string(), [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        filter.hash(&mut hasher);
        let hash_code = hasher.finish();

        assert!(hash_code != 0);
    }

    #[test]
    fn test_biquad_filter_eq() {
        let filter_1 = BiquadFilter::new("test".to_string(), [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let filter_2 = BiquadFilter::new("test".to_string(), [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);

        assert_eq!(filter_1, filter_2);
    }
}
