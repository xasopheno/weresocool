#[cfg(test)]
pub mod normalize_tests {
    extern crate num_rational;
    extern crate pretty_assertions;
    use crate::{
        ast::{Op::*, OscType, ParseTable},
        operations::{NormalForm, Normalize, PointOp},
    };
    use num_rational::{Ratio, Rational64};

    fn make_parse_table() -> ParseTable {
        ParseTable::new()
    }

    #[test]
    fn point_op_mod_by_mul() {
        let mut a = PointOp {
            fm: Ratio::new(3, 2),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(2, 1),
            g: Ratio::new(1, 2),
            l: Ratio::new(5, 2),
            osc_type: OscType::Sine,
        };

        let b = PointOp {
            fm: Ratio::new(2, 1),
            fa: Ratio::new(2, 1),
            pm: Ratio::new(1, 2),
            pa: Ratio::new(1, 2),
            g: Ratio::new(1, 2),
            l: Ratio::new(2, 1),
            osc_type: OscType::Noise,
        };

        a.mod_by(b);

        let expected = PointOp {
            fm: Ratio::new(3, 1),
            fa: Ratio::new(2, 1),
            pm: Ratio::new(1, 2),
            pa: Ratio::new(5, 2),
            g: Ratio::new(1, 4),
            l: Ratio::new(5, 2),
            osc_type: OscType::Noise,
        };

        assert_eq!(a, expected)
    }

    #[test]
    fn normal_form_mul() {
        let mut a = NormalForm::init();
        let mut b = NormalForm::init();
        let pt = make_parse_table();

        Sequence {
            operations: vec![
                TransposeM {
                    m: Rational64::new(3, 2),
                },
                TransposeM {
                    m: Rational64::new(5, 4),
                },
                Length {
                    m: Rational64::new(2, 1),
                },
            ],
        }
        .apply_to_normal_form(&mut a, &pt);

        Sequence {
            operations: vec![
                AsIs,
                TransposeA {
                    a: Rational64::new(2, 1),
                },
                Length {
                    m: Rational64::new(2, 1),
                },
            ],
        }
        .apply_to_normal_form(&mut b, &pt);

        let c = a * b;

        let expected = NormalForm {
            operations: vec![
                vec![
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        osc_type: OscType::Sine,
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        osc_type: OscType::Sine,
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(4, 1),
                        osc_type: OscType::Sine,
                    },
                ],
            ],
            length_ratio: Ratio::new(8, 1),
        };

        assert_eq!(c, expected)
    }

    #[test]
    fn normalize_asis() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        AsIs.apply_to_normal_form(&mut input, &pt);
        let expected = NormalForm {
            operations: vec![vec![PointOp::init()]],
            length_ratio: Ratio::new(1, 1),
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_sine_and_noise() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        Noise.apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Noise,
            }]],
        };

        assert_eq!(input, expected);

        Sine {}.apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_tm() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        TransposeM {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_ta() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        TransposeA {
            a: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(2, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_pan_m() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        PanM {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(2, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_pan_a() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        PanA {
            a: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(2, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_gain() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        Gain {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(2, 1),
                l: Ratio::new(1, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_silence() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        Silence {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(0, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: Ratio::new(2, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_length() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        Length {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(2, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_compose() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();

        Compose {
            operations: vec![
                TransposeM {
                    m: Ratio::new(2, 1),
                },
                Length {
                    m: Ratio::new(2, 1),
                },
            ],
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(2, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_sequence() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();

        Sequence {
            operations: vec![
                TransposeM {
                    m: Ratio::new(2, 1),
                },
                Length {
                    m: Ratio::new(2, 1),
                },
            ],
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(3, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(2, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    osc_type: OscType::Sine,
                },
            ]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_overlay() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();

        Overlay {
            operations: vec![
                TransposeM {
                    m: Ratio::new(2, 1),
                },
                Length {
                    m: Ratio::new(2, 1),
                },
            ],
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![
                vec![
                    PointOp {
                        fm: Ratio::new(2, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                    },
                ],
                vec![PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    osc_type: OscType::Sine,
                }],
            ],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_with_lr_of() {
        let pt = make_parse_table();
        let mut input = NormalForm::init();

        TransposeM {
            m: Ratio::new(3, 2),
        }
        .apply_to_normal_form(&mut input, &pt);

        WithLengthRatioOf {
            with_length_of: Box::new(Sequence {
                operations: vec![
                    Length {
                        m: Ratio::new(2, 1),
                    },
                    Length {
                        m: Ratio::new(4, 1),
                    },
                    Length {
                        m: Ratio::new(3, 1),
                    },
                ],
            }),
            main: Box::new(TransposeM {
                m: Ratio::new(2, 1),
            }),
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(9, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(3, 2),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(9, 1),
                osc_type: OscType::Sine,
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_invert() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();

        Compose {
            operations: vec![
                Sequence {
                    operations: vec![
                        TransposeM {
                            m: Ratio::new(1, 1),
                        },
                        TransposeM {
                            m: Ratio::new(9, 8),
                        },
                        TransposeM {
                            m: Ratio::new(5, 4),
                        },
                    ],
                },
                FInvert,
            ],
        }
        .apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(3, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                },
                PointOp {
                    fm: Ratio::new(8, 9),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                },
                PointOp {
                    fm: Ratio::new(4, 5),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                },
            ]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_modulate_by() {
        let pt = make_parse_table();
        let mut input = NormalForm::init();
        Sequence {
            operations: vec![
                TransposeM {
                    m: Ratio::new(1, 1),
                },
                TransposeM {
                    m: Ratio::new(9, 8),
                },
                TransposeM {
                    m: Ratio::new(5, 4),
                },
            ],
        }
        .apply_to_normal_form(&mut input, &pt);

        let modulator = ModulateBy {
            operations: vec![
                Gain {
                    m: Ratio::new(1, 1),
                },
                Gain {
                    m: Ratio::new(1, 2),
                },
            ],
        };

        modulator.apply_to_normal_form(&mut input, &pt);

        let expected = NormalForm {
            length_ratio: Ratio::new(3, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                },
                PointOp {
                    fm: Ratio::new(9, 8),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 2),
                    osc_type: OscType::Sine,
                },
                PointOp {
                    fm: Ratio::new(9, 8),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 2),
                    l: Ratio::new(1, 2),
                    osc_type: OscType::Sine,
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 2),
                    l: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                },
            ]],
        };

        assert_eq!(input, expected);
    }

}
