mod tests {
    use event::{Event, Sound};
    use instrument::oscillator::OscType;

    #[test]
    fn test_event() {
        let length = 0.001;
        let sounds = vec![Sound {
            frequency: 100.0,
            gain: 1.0,
            pan: 1.0,
            osc_type: OscType::Sine,
        }];

        let result = Event::init(100.0, 1.0, 1.0, 0.001);
        let expected = Event {
            sounds: sounds.clone(),
            length,
        };

        assert_eq!(expected, result);
    }
}
