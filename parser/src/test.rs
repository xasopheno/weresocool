pub mod test {
    extern crate num_rational;
    extern crate indexmap;
    use indexmap::IndexMap;
    use num_rational::{Ratio, Rational};
    use socool_parser::ast::{Op};
    use socool_parser::parser::*;

    fn mock_init() -> (String) {
        "{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
            main = {"
            .to_string()
    }

    fn test_parsed_operation(mut parse_str: String, expected: Op) {
        let mut table = IndexMap::new();

        parse_str.push_str("}");

        let _result = socool::SoCoolParser::new().parse(&mut table, &parse_str);

        let main = table.get(&"main".to_string()).unwrap();
        assert_eq!(*main, expected);
    }

    #[test]
    fn init_test() {
        let mut parse_str = mock_init();
        let mut table = IndexMap::new();
        parse_str.push_str("AsIs }");
        let init = socool::SoCoolParser::new()
            .parse(&mut table, &parse_str)
            .unwrap();
        assert_eq!(
            init,
            Init {
                f: Ratio::from_integer(220),
                l: Ratio::from_integer(1),
                g: Ratio::from_integer(1),
                p: Ratio::from_integer(0),
            }
        );
    }

    #[test]
    fn tm_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Tm 3/2");
        test_parsed_operation(parse_str, Op::TransposeM { m: Ratio::new(3,2)});
    }

    #[test]
    fn ta_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Ta 2.0");
        test_parsed_operation(parse_str, Op::TransposeA { a: Ratio::new(2, 1) });
    }

    #[test]
    fn pan_a_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("PanA 2.0");
        test_parsed_operation(parse_str, Op::PanA { a: Ratio::new(2, 1)  });
    }

    #[test]
    fn pan_m_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("PanM 3.0/2.0");
        test_parsed_operation(parse_str, Op::PanM { m: Ratio::new(3, 2)  });
    }

    #[test]
    fn gain_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Gain 0.25");
        test_parsed_operation(parse_str, Op::Gain { m: Ratio::new(1, 4)  });
    }

    #[test]
    fn length_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Length 0.5");
        test_parsed_operation(parse_str, Op::Length { m: Ratio::new(1, 2)  });
    }

    #[test]
    fn reverse_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Reverse");
        test_parsed_operation(parse_str, Op::Reverse);
    }

    #[test]
    fn asis_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("AsIs");
        test_parsed_operation(parse_str, Op::AsIs);
    }

    #[test]
    fn sequence_test() {
        let mut parse_str = mock_init();
        parse_str.push_str(
            "
                Sequence [
                    AsIs,
                    Tm 3/2,
                ]
            ",
        );
        test_parsed_operation(
            parse_str,
            Op::Sequence {
                operations: vec![Op::AsIs, Op::TransposeM { m: Ratio::new(3, 2)  }],
            },
        );
    }

    #[test]
    fn overlay_test() {
        let mut parse_str = mock_init();
        parse_str.push_str(
            "
                Overlay [
                    AsIs,
                    Tm 3/2,
                ]
            ",
        );
        test_parsed_operation(
            parse_str,
            Op::Overlay {
                operations: vec![Op::AsIs, Op::TransposeM { m: Ratio::new(3, 2)  }],
            },
        );
    }

    #[test]
    fn o_test() {
        let mut parse_str = mock_init();
        parse_str.push_str(
            "
                O[(3/2, 3.0, 1.0, 0.3),
                  (1, 0.0, 0.5, 0.0)]
            ",
        );
        test_parsed_operation(
            parse_str,
            Op::Overlay {
                operations: vec![
                    Op::Compose {
                        operations: vec![
                            Op::TransposeM { m: Ratio::new(3, 2)  },
                            Op::TransposeA { a: Ratio::new(3, 1)  },
                            Op::Gain { m: Ratio::new(1, 1)  },
                            Op::PanA { a: Ratio::new(3, 10)  },
                        ],
                    },
                    Op::Compose {
                        operations: vec![
                            Op::TransposeM { m: Ratio::new(1, 1)  },
                            Op::TransposeA { a: Ratio::new(0, 1)  },
                            Op::Gain { m: Ratio::new(1, 2)  },
                            Op::PanA { a: Ratio::new(0, 1)  },
                        ],
                    },
                ],
            },
        );
    }

    #[test]
    fn let_insert() {
        let mut table = IndexMap::new();
        socool::SoCoolParser::new()
            .parse(
                &mut table,
                "
                    { f: 200, l: 1.0, g: 1.0, p: 0.0 }

                    thing = {
                        Tm 3/2
                        | Gain 0.3
                    }
                    ",
            )
            .unwrap();
        let thing = table.get(&"thing".to_string()).unwrap();
        assert_eq!(
            *thing,
            Op::Compose {
                operations: vec![Op::TransposeM { m: Ratio::new(3, 2)  }, Op::Gain { m: Ratio::new(3, 10)  }]
            }
        )
    }

    #[test]
    fn let_get() {
        let mut table = IndexMap::new();
        socool::SoCoolParser::new()
            .parse(
                &mut table,
                "
                    { f: 200, l: 1.0, g: 1.0, p: 0.0 }

                    thing = {
                        Tm 3/2
                        | Gain 0.3
                    }

                    main = { thing }
                    ",
            )
            .unwrap();
    }

    #[test]
    fn fit_length_test() {
        let mut table = IndexMap::new();

        let _result = socool::SoCoolParser::new().parse(
            &mut table,
            "
                { f: 200, l: 1.0, g: 1.0, p: 0.0 }

                thing = {
                    Sequence [
                     AsIs,
                     Tm 3/2
                     | Length 2.0
                    ]
                }

                thing2 = {
                    Sequence [
                        Tm 5/4,
                        Tm 3/2
                    ]
                    | Repeat 2
                    > FitLength thing
                }

                main = {
                    thing2
                }
            ",
        );
        let thing = table.get(&"main".to_string()).unwrap();
        assert_eq!(
            *thing,
            Op::Compose {
                operations: vec![
                    Op::Compose {
                        operations: vec![
                            Op::Sequence {
                                operations: vec![
                                    Op::TransposeM { m: Ratio::new(5, 4)  },
                                    Op::TransposeM { m: Ratio::new(3, 2)  }
                                ]
                            },
                            Op::Sequence {
                                operations: vec![Op::AsIs, Op::AsIs]
                            }
                        ]
                    },
                    Op::WithLengthRatioOf {
                        with_length_of: Box::new(Op::Sequence {
                            operations: vec![
                                Op::AsIs,
                                Op::Compose {
                                    operations: vec![
                                        Op::TransposeM { m: Ratio::new(3, 2)  },
                                        Op::Length { m: Ratio::new(2, 1)  }
                                    ]
                                }
                            ]
                        }),
                        main: Box::new(Op::Compose {
                            operations: vec![
                                Op::Sequence {
                                    operations: vec![
                                        Op::TransposeM { m: Ratio::new(5, 4)  },
                                        Op::TransposeM { m: Ratio::new(3, 2)  }
                                    ]
                                },
                                Op::Sequence {
                                    operations: vec![Op::AsIs, Op::AsIs]
                                }
                            ]
                        })
                    }
                ]
            }
        )
    }
}
