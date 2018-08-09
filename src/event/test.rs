mod tests {
    use event::{Event, Render};
    use instrument::oscillator::Oscillator;
    use ratios::{Pan, R};
    use settings::get_test_settings;

    fn test_ratios() -> Vec<R> {
        r![(1, 1, 0.0, 0.5, 0.0)]
    }

    #[test]
    fn test_event() {
        let mut result = Event::new(100.0, test_ratios(), 0.001, 1.0);

        let mut expected = Event {
            frequency: 100.0,
            ratios: vec![
                R::atio(1, 1, 0.0, 0.25, Pan::Left),
                R::atio(1, 1, 0.0, 0.25, Pan::Right),
            ],
            length: 0.001,
            gain: 1.0,
        };

        assert_eq!(expected, result);
    }
}
