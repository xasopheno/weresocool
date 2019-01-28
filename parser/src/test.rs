pub mod test {
    extern crate num_rational;
    extern crate socool_ast;

    use num_rational::Ratio;
    use socool_ast::ast::{Op, OpOrNf, OpOrNfTable};
    use socool_parser::imports::{get_filepath_and_import_name, is_as_import, is_import};
    use socool_parser::parser::*;

    fn mock_init() -> (String) {
        "{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
            main = {"
            .to_string()
    }

    fn test_parsed_operation(mut parse_str: String, expected: Op) {
        let mut table = OpOrNfTable::new();

        parse_str.push_str("}");

        let _result = socool::SoCoolParser::new().parse(&mut table, &parse_str);

        let main = table.get(&"main".to_string()).unwrap();
        assert_eq!(*main, OpOrNf::Op(expected));
    }

    fn test_data() -> Vec<String> {
        let import_str = "   import standard ".to_string();
        let import_as_str = "import  standard as std".to_string();
        let not_import_as_str = "import standardasstd  ".to_string();
        let not_import = " not an  import".to_string();
        vec![import_str, import_as_str, not_import_as_str, not_import]
    }

    #[test]
    fn test_import_strings() {
        let lines = test_data();
        let starts_with_import: Vec<bool> = lines
            .iter()
            .map(|line| is_import(line.to_string()))
            .collect();

        let is_as_import: Vec<bool> = lines
            .iter()
            .map(|line| is_as_import(line.to_string()))
            .collect();

        assert_eq!(starts_with_import, vec![true, true, true, false]);
        assert_eq!(is_as_import, vec![false, true, false, false]);
    }

    #[test]
    fn test_get_filename_and_import_name() {
        let tests = vec![
            "import songs/wip/test.socool  as other_name".to_string(),
            "import ../songs/test.socool  ".to_string(),
            "import test.socool".to_string(),
        ];

        let result: Vec<(String, String)> = tests
            .iter()
            .map(|test| get_filepath_and_import_name(test.to_string()))
            .collect();

        let expected: Vec<(String, String)> = vec![
            ("songs/wip/test.socool", "other_name"),
            ("../songs/test.socool", "test"),
            ("test.socool", "test"),
        ]
        .iter()
        .map(|(a, b)| ((a.to_string(), b.to_string())))
        .collect();

        assert_eq!(result, expected)
    }

    #[test]
    fn init_test() {
        let mut parse_str = mock_init();
        let mut table = OpOrNfTable::new();
        parse_str.push_str("AsIs }");
        let init = socool::SoCoolParser::new()
            .parse(&mut table, &parse_str)
            .unwrap();
        assert_eq!(
            init,
            Init {
                f: Ratio::from_integer(200),
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
        test_parsed_operation(
            parse_str,
            Op::TransposeM {
                m: Ratio::new(3, 2),
            },
        );
    }

    #[test]
    fn ta_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Ta 2.0");
        test_parsed_operation(
            parse_str,
            Op::TransposeA {
                a: Ratio::new(2, 1),
            },
        );
    }

    #[test]
    fn pan_a_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("PanA 2.0");
        test_parsed_operation(
            parse_str,
            Op::PanA {
                a: Ratio::new(2, 1),
            },
        );
    }

    #[test]
    fn pan_m_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("PanM 3/2");
        test_parsed_operation(
            parse_str,
            Op::PanM {
                m: Ratio::new(3, 2),
            },
        );
    }

    #[test]
    fn gain_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Gain 0.25");
        test_parsed_operation(
            parse_str,
            Op::Gain {
                m: Ratio::new(1, 4),
            },
        );
    }

    #[test]
    fn length_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Length 0.5");
        test_parsed_operation(
            parse_str,
            Op::Length {
                m: Ratio::new(1, 2),
            },
        );
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
                operations: vec![
                    OpOrNf::Op(Op::AsIs),
                    OpOrNf::Op(Op::TransposeM {
                        m: Ratio::new(3, 2),
                    }),
                ],
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
                operations: vec![
                    OpOrNf::Op(Op::AsIs),
                    OpOrNf::Op(Op::TransposeM {
                        m: Ratio::new(3, 2),
                    }),
                ],
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
                    OpOrNf::Op(Op::Compose {
                        operations: vec![
                            OpOrNf::Op(Op::TransposeM {
                                m: Ratio::new(3, 2),
                            }),
                            OpOrNf::Op(Op::TransposeA {
                                a: Ratio::new(3, 1),
                            }),
                            OpOrNf::Op(Op::Gain {
                                m: Ratio::new(1, 1),
                            }),
                            OpOrNf::Op(Op::PanA {
                                a: Ratio::new(3, 10),
                            }),
                        ],
                    }),
                    OpOrNf::Op(Op::Compose {
                        operations: vec![
                            OpOrNf::Op(Op::TransposeM {
                                m: Ratio::new(1, 1),
                            }),
                            OpOrNf::Op(Op::TransposeA {
                                a: Ratio::new(0, 1),
                            }),
                            OpOrNf::Op(Op::Gain {
                                m: Ratio::new(1, 2),
                            }),
                            OpOrNf::Op(Op::PanA {
                                a: Ratio::new(0, 1),
                            }),
                        ],
                    }),
                ],
            },
        );
    }

    #[test]
    fn let_insert() {
        let mut table = OpOrNfTable::new();
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
            OpOrNf::Op(Op::Compose {
                operations: vec![
                    OpOrNf::Op(Op::TransposeM {
                        m: Ratio::new(3, 2)
                    }),
                    OpOrNf::Op(Op::Gain {
                        m: Ratio::new(3, 10)
                    })
                ]
            })
        )
    }

    //        #[test]
    //        fn let_get() {
    //            let mut table = OpOrNfTable::new();
    //            socool::SoCoolParser::new()
    //                .parse(
    //                    &mut table,
    //                    "
    //                        { f: 200, l: 1.0, g: 1.0, p: 0.0 }
    //
    //                        thing = {
    //                            Tm 3/2
    //                            | Gain 0.3
    //                        }
    //
    //                        main = { thing }
    //                        ",
    //                )
    //                .unwrap();
    //        }

    #[test]
    fn fit_length_test() {
        let mut table = OpOrNfTable::new();

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

                            main = {
                                Sequence [
                                    Tm 5/4,
                                    Tm 3/2
                                ]
                                | Repeat 2
                                > FitLength thing
                            }
                        ",
        );
        let thing = table.get(&"main".to_string()).unwrap();
        assert_eq!(
            *thing,
            OpOrNf::Op(Op::Compose {
                operations: vec![
                    OpOrNf::Op(Op::Compose {
                        operations: vec![
                            OpOrNf::Op(Op::Sequence {
                                operations: vec![
                                    OpOrNf::Op(Op::TransposeM {
                                        m: Ratio::new(5, 4)
                                    }),
                                    OpOrNf::Op(Op::TransposeM {
                                        m: Ratio::new(3, 2)
                                    })
                                ]
                            }),
                            OpOrNf::Op(Op::Sequence {
                                operations: vec![OpOrNf::Op(Op::AsIs), OpOrNf::Op(Op::AsIs)]
                            })
                        ]
                    }),
                    OpOrNf::Op(Op::WithLengthRatioOf {
                        with_length_of: Box::new(OpOrNf::Op(Op::Id(vec!["thing".to_string()]))),
                        main: Box::new(OpOrNf::Op(Op::Compose {
                            operations: vec![
                                OpOrNf::Op(Op::Sequence {
                                    operations: vec![
                                        OpOrNf::Op(Op::TransposeM {
                                            m: Ratio::new(5, 4)
                                        }),
                                        OpOrNf::Op(Op::TransposeM {
                                            m: Ratio::new(3, 2)
                                        })
                                    ]
                                }),
                                OpOrNf::Op(Op::Sequence {
                                    operations: vec![OpOrNf::Op(Op::AsIs), OpOrNf::Op(Op::AsIs)]
                                })
                            ]
                        }))
                    })
                ]
            })
        )
    }
}
