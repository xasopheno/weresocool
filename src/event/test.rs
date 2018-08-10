mod tests {
    use event::{Event};
    use ratios::{Pan, R};

    fn test_ratios() -> Vec<R> {
        r![(1, 1, 0.0, 0.5, 0.0)]
    }

    #[test]
    fn test_event() {
        let result = Event::new(100.0, test_ratios(), 0.001, 1.0);

        let expected = Event {
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
