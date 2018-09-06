pub mod tests {
    use operations::Op;
    #[test]
    fn test_r_macro() {
        let r_macro = r![
            (1, 1, 3.0, 1.0, 1.0),
            (3, 2, 0.0, 0.6, -1.0),
            (5, 4, 1.5, 0.5, 0.5),
        ];

        let macro_test = Op::Overlay {
            operations: vec![
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 1.0 / 1.0 },
                        Op::TransposeA { a: 3.0 },
                        Op::Gain { m: 1.0 },
                        Op::PanA { a: 1.0 },
                    ],
                },
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 3.0 / 2.0 },
                        Op::TransposeA { a: 0.0 },
                        Op::Gain { m: 0.6 },
                        Op::PanA { a: -1.0 },
                    ],
                },
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 5.0 / 4.0 },
                        Op::TransposeA { a: 1.5 },
                        Op::Gain { m: 0.5 },
                        Op::PanA { a: 0.5 },
                    ],
                },
            ],
        };

        assert_eq!(r_macro, macro_test);
    }
    #[test]
    fn test_sequence_macro() {
        let sequence = sequence![
                Op::AsIs,
                Op::TransposeM { m: 2.0 }
            ];

        let expected = Op::Sequence {
            operations: { vec![
                Op::AsIs {},
                Op::TransposeM {m: 2.0}
            ]
            }};

        assert_eq!(sequence, expected);
    }
    #[test]
    fn test_overlay_macro() {
        let overlay = overlay![
                Op::AsIs,
                Op::TransposeM { m: 2.0 }
            ];

        let expected = Op::Overlay {
            operations: { vec![
                Op::AsIs,
                Op::TransposeM {m: 2.0}
            ]
            }};

        assert_eq!(overlay, expected);
    }

    #[test]
    fn test_compose_macro() {
        let compose = compose![
                Op::AsIs,
                Op::TransposeM { m: 2.0 }
            ];

        let expected = Op::Compose {
            operations: { vec![
                Op::AsIs {},
                Op::TransposeM {m: 2.0}
            ]
            }};

        assert_eq!(compose, expected);
    }

    #[test]
    fn test_fit_macro() {
        let compose = fit![
                Op::AsIs => Op::AsIs, 4
            ];

        let expected = Op::Fit {
            n: 4,
            with_length_of: Box::new(Op::AsIs),
            main: Box::new(Op::AsIs),
        };

        assert_eq!(compose, expected);
    }
}
