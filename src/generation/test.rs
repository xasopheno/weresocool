#[cfg(test)]
#[allow(clippy::unreadable_literal)]
pub mod tests {
    use crate::generation::{
        composition_to_vec_timed_op, sum_vec, vec_timed_op_to_vec_op4d, EventType, Op4D, TimedOp,
    };
    use num_rational::Rational64;
    use pretty_assertions::assert_eq;
    use scop::Defs;
    use weresocool_ast::{NameSet, NormalForm, Normalize, Op::*, OscType, Term, Term::Op, ASR};
    use weresocool_instrument::Basis;
    use weresocool_shared::helpers::cmp_vec_f64;

    #[test]
    fn render_equal() {
        let mut a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        sum_vec(&mut a, &b[..]);
        let expected = [2.0, 4.0, 6.0];
        assert!(cmp_vec_f64(a.to_vec(), expected.to_vec()));
    }

    #[test]
    fn render_left() {
        let mut a = vec![1.0, 2.0, 3.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        sum_vec(&mut a, &b[..]);
        let expected = [2.0, 4.0, 6.0, 2.0];
        assert!(cmp_vec_f64(a.to_vec(), expected.to_vec()));
    }

    #[test]
    fn to_vec_timed_op_test() {
        let mut normal_form = NormalForm::init();
        let mut pt: Defs<Term> = Default::default();

        Overlay {
            operations: vec![
                Op(Sequence {
                    operations: vec![
                        Op(PanA {
                            a: Rational64::new(1, 2),
                        }),
                        Op(TransposeM {
                            m: Rational64::new(2, 1),
                        }),
                        Op(Gain {
                            m: Rational64::new(1, 2),
                        }),
                        Op(Length {
                            m: Rational64::new(2, 1),
                        }),
                    ],
                }),
                Op(Sequence {
                    operations: vec![Op(Length {
                        m: Rational64::new(5, 1),
                    })],
                }),
            ],
        }
        .apply_to_normal_form(&mut normal_form, &mut pt)
        .unwrap();

        let timed_ops = composition_to_vec_timed_op(&normal_form, &mut pt).unwrap();

        let op = TimedOp {
            fm: Rational64::new(1, 1),
            fa: Rational64::new(0, 1),
            pm: Rational64::new(1, 1),
            pa: Rational64::new(0, 1),
            g: Rational64::new(1, 1),
            l: Rational64::new(1, 1),
            t: Rational64::new(0, 1),
            reverb: Rational64::new(0, 1),
            event_type: EventType::On,
            voice: 0,
            event: 0,
            attack: Rational64::new(1, 1),
            decay: Rational64::new(1, 1),
            asr: ASR::Long,
            portamento: Rational64::new(1, 1),
            osc_type: OscType::None,
            names: vec![],
        };

        assert_eq!(
            timed_ops,
            (
                vec![
                    TimedOp {
                        pa: Rational64::new(1, 2),
                        event_type: EventType::On,
                        ..op.clone()
                    },
                    TimedOp {
                        event_type: EventType::On,
                        l: Rational64::new(5, 1),
                        voice: 1,
                        ..op.clone()
                    },
                    TimedOp {
                        fm: Rational64::new(2, 1),
                        t: Rational64::new(1, 1),
                        event_type: EventType::On,
                        event: 1,
                        ..op.clone()
                    },
                    TimedOp {
                        g: Rational64::new(1, 2),
                        t: Rational64::new(2, 1),
                        event_type: EventType::On,
                        event: 2,
                        ..op.clone()
                    },
                    TimedOp {
                        t: Rational64::new(3, 1),
                        l: Rational64::new(2, 1),
                        event_type: EventType::On,
                        event: 3,
                        ..op.clone()
                    },
                ],
                2
            )
        );
    }

    #[test]
    fn to_vec_op4d_test() {
        let basis = Basis {
            f: Rational64::new(100, 1),
            g: Rational64::new(1, 1),
            p: Rational64::new(0, 1),
            l: Rational64::new(1, 1),
            a: Rational64::new(1, 1),
            d: Rational64::new(1, 1),
        };

        let op = TimedOp {
            fm: Rational64::new(2, 1),
            fa: Rational64::new(0, 1),
            pm: Rational64::new(1, 1),
            pa: Rational64::new(1, 2),
            g: Rational64::new(1, 2),
            t: Rational64::new(0, 1),
            l: Rational64::new(1, 1),
            reverb: Rational64::new(0, 1),
            event_type: EventType::On,
            voice: 0,
            event: 0,
            attack: Rational64::new(1, 1),
            decay: Rational64::new(1, 1),
            asr: ASR::Short,
            portamento: Rational64::new(1, 1),
            osc_type: OscType::None,
            names: vec![],
        };

        let vec_timed_op = vec![
            TimedOp {
                event_type: EventType::On,
                l: Rational64::new(3, 2),
                ..op.clone()
            },
            TimedOp {
                event_type: EventType::Off,
                l: Rational64::new(3, 2),
                t: Rational64::new(3, 2),
                ..op.clone()
            },
        ];

        let result = vec_timed_op_to_vec_op4d(vec_timed_op, &basis);
        let expected = vec![
            Op4D {
                t: 0.0,
                l: 1.5,
                event_type: EventType::On,
                voice: 0,
                event: 0,
                y: 2.3010299956639813,
                x: 0.5,
                z: 0.5,
                names: vec![],
            },
            Op4D {
                t: 1.5,
                l: 1.5,
                event_type: EventType::Off,
                voice: 0,
                event: 0,
                x: 0.5,
                y: 2.3010299956639813,
                z: 0.5,
                names: vec![],
            },
        ];
        assert_eq!(result, expected);
    }
}

//mod parse_tests {
//    extern crate socool_parser;
//    #[test]
//    fn import_test() {
//        use socool_parser::parse_file;
//        let filename = &"songs/test/import_test.socool".to_string();
//        let parsed = parse_file(filename, None);
//        let mut result: Vec<String> = parsed
//            .table
//            .iter()
//            .map(|(name, _)| name.to_string())
//            .collect();
//
//        result.sort();
//
//        let mut expected: Vec<String> = vec![
//            "import_test_2.main",
//            "import_test_2.std_test.fade_out",
//            "import_test_2.std_test.main",
//            "import_test_2.thing",
//            "main",
//            "standard.fade_out",
//            "standard.import_test_2.main",
//            "standard.import_test_2.std_test.fade_out",
//            "standard.import_test_2.std_test.main",
//            "standard.import_test_2.thing",
//            "standard.main",
//            "thing",
//        ]
//        .iter()
//        .map(|s| s.to_string())
//        .collect();
//
//        expected.sort();
//
//        assert_eq!(expected, result)
//    }
//}
