pub mod tests {
    use operations::{Op, Operate};
    use event::{Sound, Event};

    fn event1() -> Event {
        Event {
            sounds: vec![
                Sound {
                    frequency: 100.0,
                    gain: 1.0,
                    pan: 0.0
                }
            ],
            length: 1.0
        }
    }

    fn event2() -> Event {
        Event {
            sounds: vec![
                Sound {
                    frequency: 100.0,
                    gain: 1.0,
                    pan: 0.0
                }
            ],
            length: 1.0
        }
    }

    fn vec_event1() -> Vec<Event> {
        vec![event1(), event2()]
    }

    fn sequence1() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::AsIs,
                Op::TransposeM { m: 2.0 },
                Op::Length { m: 2.0 },
            ],
        }
    }

    fn sequence2() -> Op {
        Op::Compose {
            operations: vec![sequence1(), Op::Length { m: 2.0 }],
        }
    }

    fn sequence3() -> Op {
        Op::Compose {
            operations: vec![sequence1(), sequence2()],
        }
    }

    #[test]
    fn op_asis_test() {
        let as_is = Op::AsIs {};
        assert_eq!(as_is.get_length_ratio(), 1.0);
        assert_eq!(as_is.apply(vec_event1()), vec_event1())
    }

    #[test]
    fn op_reverse_test() {
        let reverse = Op::Reverse {};
        assert_eq!(reverse.get_length_ratio(), 1.0);
        let apply_expected = vec![
            event2(),
            event1(),
        ];
        assert_eq!(reverse.apply(vec_event1()), apply_expected);
    }

    #[test]
    fn op_pan_test() {
        let pan = Op::Pan { a: 0.5 };
        assert_eq!(pan.get_length_ratio(), 1.0);
        let mut expected_event = event1();
        expected_event.sounds[0].pan = 0.5;
        let pan_apply_expected = vec![expected_event];

        assert_eq!(pan.apply(vec![event1()]), pan_apply_expected);
    }

    #[test]
    fn op_repeat_test() {
        let sequence1 = sequence1();
        let repeat = Op::Repeat {
            n: 2,
            operations: vec![sequence1.clone()],
        }.get_length_ratio();

        assert_eq!(repeat, 10.0);
    }

    #[test]
    fn op_length_test() {
        let length = Op::Length { m: 1.5 };
        assert_eq!(length.get_length_ratio(), 1.5);
    }

    #[test]
    fn op_transpose_m_test() {
        let transpose_m = Op::TransposeM { m: 1.5 };
        assert_eq!(transpose_m.get_length_ratio(), 1.0);
    }

    #[test]
    fn op_transpose_a_test() {
        let transpose_a = Op::TransposeA { a: 1.5 };
        assert_eq!(transpose_a.get_length_ratio(), 1.0);
    }

    #[test]
    fn op_silence_test() {
        let silence = Op::Silence { m: 1.5 };
        assert_eq!(silence.get_length_ratio(), 1.5);
    }

    #[test]
    fn op_gain_test() {
        let gain = Op::Gain { m: 1.5 };
        assert_eq!(gain.get_length_ratio(), 1.0);
    }

    #[test]
    fn op_fit() {
        let fit = Op::Fit {
            n: 1,
            with_length_of: Box::new(sequence1()),
            main: Box::new(sequence3()),
        };

        let fit_length = fit.get_length_ratio();


        assert_eq!(fit_length, 5.0);
    }

    #[test]
    fn op_compose_test() {
        let sequence_with_sequence_length = sequence3().get_length_ratio();
        assert_eq!(sequence_with_sequence_length, 50.0);
    }

    #[test]
    fn op_sequence_test() {
        let sequence_length_1 = sequence1().get_length_ratio();
        let sequence_length_2 = sequence2().get_length_ratio();
        assert_eq!(sequence_length_1, 5.0);
        assert_eq!(sequence_length_2, 10.0);
    }

    #[test]
    fn op_overlay_test() {
        let overlay = Op::Overlay {
            operations: vec![
                sequence1(),
                sequence2(),
                sequence3()
            ]
        };
        assert_eq!(overlay.get_length_ratio(), 50.0);
    }
}
