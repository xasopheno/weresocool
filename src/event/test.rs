mod tests {
    use event::{Event, Mutate, Phrase, Render};
    use oscillator::Oscillator;
    use ratios::{Pan, R};
    use settings::get_test_settings;

    fn test_ratios() -> Vec<R> {
        r![(1, 1, 0.0, 0.5, 1.0), (1, 1, 1.0, 0.5, -1.0)]
    }

    fn test_ratios_change() -> Vec<R> {
        r![(3, 2, 0.0, 0.6, -1.0), (5, 4, 1.5, 0.5, 0.5)]
    }

    #[test]
    fn test_event() {
        let mut result = Event::new(100.0, test_ratios(), 0.001, 1.0)
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 0.0)
            .mut_gain(0.9, 0.0);

        let mut expected = Event {
            frequency: 150.0,
            ratios: vec![
                R::atio(3, 2, 0.0, 0.6, Pan::Left),
                R::atio(3, 2, 0.0, 0.0, Pan::Right),
                R::atio(5, 4, 1.5, 0.125, Pan::Left),
                R::atio(5, 4, 1.5, 0.375, Pan::Right),
            ],
            length: 0.002,
            gain: 0.9,
        };
        let mut oscillator1 = Oscillator::init(test_ratios(), &get_test_settings());
        let mut oscillator2 = Oscillator::init(test_ratios(), &get_test_settings());

        assert_eq!(expected, result);
        assert_eq!(
            expected.render(&mut oscillator1),
            result.render(&mut oscillator2)
        );
    }

    #[test]
    fn test_phrase() {
        let mut phrase = Phrase {
            events: vec![
                Event::new(100.0, test_ratios(), 1.0, 1.0),
                Event::new(50.0, test_ratios(), 2.0, 1.0),
            ],
        };

        let mut oscillator1 = Oscillator::init(test_ratios(), &get_test_settings());
        let mut result = phrase
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let mut oscillator2 = Oscillator::init(test_ratios(), &get_test_settings());
        let mut expected = Phrase {
            events: vec![
                Event::new(150.0, test_ratios_change(), 3.0, 0.9),
                Event::new(75.0, test_ratios_change(), 5.0, 0.9),
            ],
        };
        assert_eq!(result, expected);
        assert_eq!(
            result.render(&mut oscillator1),
            expected.render(&mut oscillator2)
        );
    }

    #[test]
    fn test_vec_phrases() {
        let phrase1 = Phrase {
            events: vec![
                Event::new(50.0, test_ratios(), 1.0, 1.0),
                Event::new(50.0, test_ratios(), 1.0, 1.0),
            ],
        };

        let phrase2 = Phrase {
            events: vec![
                Event::new(100.0, test_ratios(), 1.0, 1.0),
                Event::new(100.0, test_ratios(), 2.0, 1.0),
            ],
        };

        let vec_phrases = vec![phrase1, phrase2];

        let mut oscillator1 = Oscillator::init(test_ratios(), &get_test_settings());
        let mut result = vec_phrases
            .clone()
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let mut oscillator2 = Oscillator::init(test_ratios(), &get_test_settings());
        let mut expected = vec![
            Phrase {
                events: vec![
                    Event::new(
                        75.0,
                        vec![
                            R::atio(3, 2, 0.0, 0.6, Pan::Left),
                            R::atio(3, 2, 0.0, 0.0, Pan::Right),
                            R::atio(5, 4, 1.5, 0.125, Pan::Left),
                            R::atio(5, 4, 1.5, 0.375, Pan::Right),
                        ],
                        3.0,
                        0.9,
                    ),
                    Event::new(
                        75.0,
                        vec![
                            R::atio(3, 2, 0.0, 0.6, Pan::Left),
                            R::atio(3, 2, 0.0, 0.0, Pan::Right),
                            R::atio(5, 4, 1.5, 0.125, Pan::Left),
                            R::atio(5, 4, 1.5, 0.375, Pan::Right),
                        ],
                        3.0,
                        0.9,
                    ),
                ],
            },
            Phrase {
                events: vec![
                    Event::new(
                        150.0,
                        vec![
                            R::atio(3, 2, 0.0, 0.6, Pan::Left),
                            R::atio(3, 2, 0.0, 0.0, Pan::Right),
                            R::atio(5, 4, 1.5, 0.125, Pan::Left),
                            R::atio(5, 4, 1.5, 0.375, Pan::Right),
                        ],
                        3.0,
                        0.9,
                    ),
                    Event::new(
                        150.0,
                        vec![
                            R::atio(3, 2, 0.0, 0.6, Pan::Left),
                            R::atio(3, 2, 0.0, 0.0, Pan::Right),
                            R::atio(5, 4, 1.5, 0.125, Pan::Left),
                            R::atio(5, 4, 1.5, 0.375, Pan::Right),
                        ],
                        5.0,
                        0.9,
                    ),
                ],
            },
        ];

        assert_eq!(expected, result);
        assert_eq!(
            result.render(&mut oscillator1),
            expected.render(&mut oscillator2)
        );
    }
}
