#[cfg(test)]
pub mod tests {
    extern crate num_rational;
    extern crate pretty_assertions;
    use crate::{Defs, NameSet, NormalForm, Normalize, Op::*, OscType, PointOp, Term::*, ASR};
    use num_rational::{Ratio, Rational64};

    fn make_parse_table() -> Defs {
        Default::default()
    }

    fn mock_names() -> (NameSet, NameSet) {
        let mut names_bar = NameSet::new();
        names_bar.insert("bar".to_string());
        let mut names_foo_bar = NameSet::new();
        names_foo_bar.insert("foo".to_string());
        names_foo_bar.insert("bar".to_string());

        (names_bar, names_foo_bar)
    }

    fn mock() -> NormalForm {
        let mut a = NormalForm::init();
        let mut b = NormalForm::init();
        let mut pt = make_parse_table();

        let foo_tag = Op(Compose {
            operations: vec![
                Op(TransposeM {
                    m: Rational64::new(5, 4),
                }),
                Op(Tag("foo".to_string())),
            ],
        });

        let bar_tag = Op(Compose {
            operations: vec![
                Op(Tag("bar".to_string())),
                Op(Sequence {
                    operations: vec![
                        Op(TransposeM {
                            m: Rational64::new(3, 2),
                        }),
                        Op(Id("foo".to_string())),
                        Op(Length {
                            m: Rational64::new(2, 1),
                        }),
                    ],
                }),
            ],
        });

        pt.terms.insert("foo".to_string(), foo_tag);
        pt.terms.insert("bar".to_string(), bar_tag.clone());

        bar_tag.apply_to_normal_form(&mut a, &pt).unwrap();

        Sequence {
            operations: vec![
                Op(AsIs),
                Op(TransposeA {
                    a: Rational64::new(2, 1),
                }),
                Op(Length {
                    m: Rational64::new(2, 1),
                }),
            ],
        }
        .apply_to_normal_form(&mut b, &pt)
        .unwrap();

        a * b
    }

    #[test]
    fn normal_form_partition_named() {
        let nf = mock();
        let (named, _rest) = nf.partition("foo".to_string());
        let (_names_bar, names_foo_bar) = mock_names();

        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: names_foo_bar.clone(),
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(2, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: names_foo_bar.clone(),
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: names_foo_bar,
                },
            ]],
            length_ratio: Ratio::new(8, 1),
        };

        assert_eq!(named, expected)
    }

    #[test]
    fn normal_form_partition_rest() {
        let nf = mock();
        let (_named, rest) = nf.partition("foo".to_string());
        let (names_bar, names_foo_bar) = mock_names();

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
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_foo_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_foo_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_foo_bar,
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(4, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar,
                    },
                ],
            ],
            length_ratio: Ratio::new(8, 1),
        };

        assert_eq!(rest, expected)
    }
    #[test]
    fn point_op_mod_by_mul() {
        let mut names_a = NameSet::new();
        names_a.insert("foo".to_string());
        let mut a = PointOp {
            fm: Ratio::new(3, 2),
            fa: Ratio::new(0, 1),
            pm: Ratio::new(1, 1),
            pa: Ratio::new(2, 1),
            g: Ratio::new(1, 2),
            l: Ratio::new(5, 2),
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            asr: ASR::Long,
            portamento: Ratio::new(1, 1),
            osc_type: OscType::Sine,
            names: names_a,
        };

        let mut names_b = NameSet::new();
        names_b.insert("bar".to_string());
        let b = PointOp {
            fm: Ratio::new(2, 1),
            fa: Ratio::new(2, 1),
            pm: Ratio::new(1, 2),
            pa: Ratio::new(1, 2),
            g: Ratio::new(1, 2),
            l: Ratio::new(2, 1),
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            asr: ASR::Long,
            portamento: Ratio::new(1, 1),
            osc_type: OscType::Noise,
            names: names_b,
        };

        a.mod_by(b);

        let mut names_expected = NameSet::new();
        names_expected.insert("foo".to_string());
        names_expected.insert("bar".to_string());
        let expected = PointOp {
            fm: Ratio::new(3, 1),
            fa: Ratio::new(2, 1),
            pm: Ratio::new(1, 2),
            pa: Ratio::new(5, 2),
            g: Ratio::new(1, 4),
            l: Ratio::new(5, 2),
            attack: Ratio::new(1, 1),
            decay: Ratio::new(1, 1),
            asr: ASR::Long,
            portamento: Ratio::new(1, 1),
            osc_type: OscType::Noise,
            names: names_expected,
        };

        assert_eq!(a, expected)
    }

    #[test]
    fn normal_form_mul() {
        let (names_bar, names_foo_bar) = mock_names();

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
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_foo_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_foo_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_foo_bar,
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(2, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(2, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar.clone(),
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(1, 1),
                        l: Ratio::new(4, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: names_bar,
                    },
                ],
            ],
            length_ratio: Ratio::new(8, 1),
        };

        assert_eq!(mock(), expected)
    }

    #[test]
    fn normalize_asis() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        AsIs.apply_to_normal_form(&mut input, &pt).unwrap();
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
        Noise.apply_to_normal_form(&mut input, &pt).unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Noise,
                names: NameSet::new(),
            }]],
        };

        assert_eq!(input, expected);

        Sine {}.apply_to_normal_form(&mut input, &pt).unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_portamento() {
        let mut input = NormalForm::init();
        let pt = make_parse_table();
        Portamento {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(2, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(2, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(2, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(2, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(2, 1),
                l: Ratio::new(1, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(0, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: Ratio::new(2, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(2, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
                Op(TransposeM {
                    m: Ratio::new(2, 1),
                }),
                Op(Length {
                    m: Ratio::new(2, 1),
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(2, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
                Op(TransposeM {
                    m: Ratio::new(2, 1),
                }),
                Op(Length {
                    m: Ratio::new(2, 1),
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

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
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
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
                Op(TransposeM {
                    m: Ratio::new(2, 1),
                }),
                Op(Length {
                    m: Ratio::new(2, 1),
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

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
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: NameSet::new(),
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: Ratio::new(1, 1),
                        attack: Ratio::new(1, 1),
                        decay: Ratio::new(1, 1),
                        asr: ASR::Long,
                        portamento: Ratio::new(1, 1),
                        osc_type: OscType::Sine,
                        names: NameSet::new(),
                    },
                ],
                vec![PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
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
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        WithLengthRatioOf {
            with_length_of: Box::new(Op(Sequence {
                operations: vec![
                    Op(Length {
                        m: Ratio::new(2, 1),
                    }),
                    Op(Length {
                        m: Ratio::new(4, 1),
                    }),
                    Op(Length {
                        m: Ratio::new(3, 1),
                    }),
                ],
            })),
            main: Box::new(Op(TransposeM {
                m: Ratio::new(2, 1),
            })),
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let expected = NormalForm {
            length_ratio: Ratio::new(9, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(3, 2),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(9, 1),
                attack: Ratio::new(1, 1),
                decay: Ratio::new(1, 1),
                asr: ASR::Long,
                portamento: Ratio::new(1, 1),
                osc_type: OscType::Sine,
                names: NameSet::new(),
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
                Op(Sequence {
                    operations: vec![
                        Op(TransposeM {
                            m: Ratio::new(1, 1),
                        }),
                        Op(TransposeM {
                            m: Ratio::new(9, 8),
                        }),
                        Op(TransposeM {
                            m: Ratio::new(5, 4),
                        }),
                    ],
                }),
                Op(FInvert),
            ],
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

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
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
                PointOp {
                    fm: Ratio::new(8, 9),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
                PointOp {
                    fm: Ratio::new(4, 5),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
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
                Op(TransposeM {
                    m: Ratio::new(1, 1),
                }),
                Op(TransposeM {
                    m: Ratio::new(9, 8),
                }),
                Op(TransposeM {
                    m: Ratio::new(5, 4),
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &pt)
        .unwrap();

        let modulator = ModulateBy {
            operations: vec![
                Op(Gain {
                    m: Ratio::new(1, 1),
                }),
                Op(Gain {
                    m: Ratio::new(1, 2),
                }),
            ],
        };

        modulator.apply_to_normal_form(&mut input, &pt).unwrap();

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
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    portamento: Ratio::new(1, 1),
                    asr: ASR::Long,
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
                PointOp {
                    fm: Ratio::new(9, 8),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 2),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
                PointOp {
                    fm: Ratio::new(9, 8),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 2),
                    l: Ratio::new(1, 2),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 2),
                    l: Ratio::new(1, 1),
                    attack: Ratio::new(1, 1),
                    decay: Ratio::new(1, 1),
                    asr: ASR::Long,
                    portamento: Ratio::new(1, 1),
                    osc_type: OscType::Sine,
                    names: NameSet::new(),
                },
            ]],
        };

        assert_eq!(input, expected);
    }
}
