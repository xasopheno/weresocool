#[cfg(test)]
pub mod tests {
    extern crate num_rational;
    extern crate pretty_assertions;
    use crate::{GetLengthRatio, NameSet, NormalForm, Normalize, Op::*, OscType, PointOp, Term::*, Defs};
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

        pt.ops.insert("global", "foo", foo_tag);
        pt.ops.insert("global", "bar", bar_tag.clone());

        bar_tag.apply_to_normal_form(&mut a, &mut pt).unwrap();

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
        .apply_to_normal_form(&mut b, &mut pt)
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
                    names: names_foo_bar.clone(),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(2, 1),
                    names: names_foo_bar.clone(),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    l: Ratio::new(2, 1),
                    names: names_foo_bar,
                    ..PointOp::init()
                },
            ]],
            start_at: None,
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
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        l: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        names: names_foo_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        names: names_foo_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: Ratio::new(2, 1),
                        names: names_foo_bar,
                        ..PointOp::init()
                    },
                ],
                vec![
                    PointOp {
                        l: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fa: Ratio::new(2, 1),
                        l: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        l: Ratio::new(4, 1),
                        names: names_bar,
                        ..PointOp::init()
                    },
                ],
            ],
            start_at: None,
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
            pm: Ratio::new(1, 1),
            pa: Ratio::new(2, 1),
            g: Ratio::new(1, 2),
            l: Ratio::new(5, 2),
            names: names_a,
            ..PointOp::init()
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
            osc_type: OscType::Noise,
            names: names_b,
            ..PointOp::init()
        };

        a.mod_by(b, a.l);

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
            osc_type: OscType::Noise,
            names: names_expected,
            ..PointOp::init()
        };

        assert_eq!(a, expected)
    }

    #[test]
    fn point_op_phase_composition() {
        // Phase carries through the op algebra with "outer wins if set, else
        // carry": exactly the rule used for `reverb`/`osc_type`. This keeps
        // analyzer-seeded phase intact under composition with phase-less ops.
        let with_phase = PointOp {
            phase: Some(Ratio::new(1, 4)),
            ..PointOp::init()
        };
        let no_phase = PointOp::init();
        assert_eq!(no_phase.phase, None);

        // outer (rhs) carries the inner (lhs) phase when the outer is None
        assert_eq!((&with_phase * &no_phase).phase, Some(Ratio::new(1, 4)));
        // outer wins when it sets a phase
        let other_phase = PointOp {
            phase: Some(Ratio::new(1, 2)),
            ..PointOp::init()
        };
        assert_eq!((&with_phase * &other_phase).phase, Some(Ratio::new(1, 2)));
        // two phase-less ops stay phase-less (legacy behavior unchanged)
        assert_eq!((&no_phase * &no_phase).phase, None);

        // mod_by and MulAssign follow the same rule
        let mut m = no_phase.clone();
        m.mod_by(with_phase.clone(), m.l);
        assert_eq!(m.phase, Some(Ratio::new(1, 4)));
        let mut ma = with_phase.clone();
        ma *= no_phase.clone();
        assert_eq!(ma.phase, Some(Ratio::new(1, 4)));
    }

    #[test]
    fn normal_form_mul() {
        let (names_bar, names_foo_bar) = mock_names();

        let expected = NormalForm {
            operations: vec![
                vec![
                    PointOp {
                        fm: Ratio::new(3, 2),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        fa: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(3, 2),
                        l: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                ],
                vec![
                    PointOp {
                        fm: Ratio::new(5, 4),
                        names: names_foo_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        fa: Ratio::new(2, 1),
                        names: names_foo_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(5, 4),
                        l: Ratio::new(2, 1),
                        names: names_foo_bar,
                        ..PointOp::init()
                    },
                ],
                vec![
                    PointOp {
                        l: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fa: Ratio::new(2, 1),
                        l: Ratio::new(2, 1),
                        names: names_bar.clone(),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(1, 1),
                        l: Ratio::new(4, 1),
                        names: names_bar,
                        ..PointOp::init()
                    },
                ],
            ],
            start_at: None,
            length_ratio: Ratio::new(8, 1),
        };

        assert_eq!(mock(), expected)
    }

    #[test]
    fn normalize_asis() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        AsIs.apply_to_normal_form(&mut input, &mut pt).unwrap();
        let expected = NormalForm {
            operations: vec![vec![PointOp::init()]],
            start_at: None,
            length_ratio: Ratio::new(1, 1),
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_sine_and_noise() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        Noise.apply_to_normal_form(&mut input, &mut pt).unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                osc_type: OscType::Noise,
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);

        Sine { pow: None }
            .apply_to_normal_form(&mut input, &mut pt)
            .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                osc_type: OscType::Sine { pow: None },
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_tm() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        TransposeM {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_portamento() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        Portamento {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                portamento: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_ta() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        TransposeA {
            a: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fa: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_pan_m() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        PanM {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                pm: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_pan_a() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        PanA {
            a: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                pa: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_gain() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        Gain {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                g: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_silence() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        Silence {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_length() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        Length {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                l: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_compose() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

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
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                l: Ratio::new(2, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_sequence() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

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
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(3, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(2, 1),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    ..PointOp::init()
                },
            ]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_overlay() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

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
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(2, 1),
            operations: vec![
                vec![
                    PointOp {
                        fm: Ratio::new(2, 1),
                        ..PointOp::init()
                    },
                    PointOp {
                        fm: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        ..PointOp::init()
                    },
                ],
                vec![PointOp {
                    fm: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    ..PointOp::init()
                }],
            ],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_with_lr_of() {
        let mut pt = make_parse_table();
        let mut input = NormalForm::init();

        TransposeM {
            m: Ratio::new(3, 2),
        }
        .apply_to_normal_form(&mut input, &mut pt)
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
            main: Some(Box::new(Op(TransposeM {
                m: Ratio::new(2, 1),
                }))),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(9, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(3, 2),
                l: Ratio::new(9, 1),
                ..PointOp::init()
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_invert() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

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
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(3, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(1, 1),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(8, 9),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(4, 5),
                    ..PointOp::init()
                },
            ]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_modulate_by() {
        let mut pt = make_parse_table();
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
        .apply_to_normal_form(&mut input, &mut pt)
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
            output: None,
        };

        modulator.apply_to_normal_form(&mut input, &mut pt).unwrap();

        let expected = NormalForm {
            start_at: None,
            length_ratio: Ratio::new(3, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(1, 1),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(9, 8),
                    l: Ratio::new(1, 2),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(9, 8),
                    g: Ratio::new(1, 2),
                    l: Ratio::new(1, 2),
                    ..PointOp::init()
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    g: Ratio::new(1, 2),
                    ..PointOp::init()
                },
            ]],
        };

        assert_eq!(input, expected);
    }

    /// THE ISORHYTHM. Seven pitches under two durations: the pitches are the
    /// subject and arrive through the pipe, the lengths cycle under them, and
    /// the pattern's own `fm = 1` means it contributes nothing but length.
    /// This is the case the op exists for.
    #[test]
    fn zip_runs_a_rhythm_under_a_melody() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

        Compose {
            operations: vec![
                Op(Sequence {
                    operations: (1..=7)
                        .map(|n| {
                            Op(TransposeM {
                                m: Rational64::new(n, 1),
                            })
                        })
                        .collect(),
                }),
                Op(Zip {
                    operations: vec![
                        Op(Compose {
                            operations: vec![
                                Op(Length {
                                    m: Rational64::new(3, 1),
                                }),
                                Op(Length {
                                    m: Rational64::new(1, 5),
                                }),
                            ],
                        }),
                        Op(Compose {
                            operations: vec![
                                Op(Length {
                                    m: Rational64::new(2, 1),
                                }),
                                Op(Length {
                                    m: Rational64::new(1, 5),
                                }),
                            ],
                        }),
                    ],
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let voice = &input.operations[0];
        assert_eq!(voice.len(), 7, "the SUBJECT's event count, not an lcm");

        let pitches: Vec<Rational64> = voice.iter().map(|p| p.fm).collect();
        assert_eq!(
            pitches,
            (1..=7).map(|n| Rational64::new(n, 1)).collect::<Vec<_>>()
        );

        let lengths: Vec<Rational64> = voice.iter().map(|p| p.l).collect();
        let long = Rational64::new(3, 5);
        let short = Rational64::new(2, 5);
        assert_eq!(lengths, vec![long, short, long, short, long, short, long]);

        // 4 longs + 3 shorts
        assert_eq!(input.length_ratio, Rational64::new(18, 5));
    }

    // ── ARTICULATION ────────────────────────────────────────────────────

    /// `Gate` multiplies and `Nudge` adds, matching the `m`/`a` convention.
    /// Composition must stay UNCLAMPED: `Gate 2 | Gate 1/2` is `Gate 1`, not
    /// `Gate 1/2`. Clamping belongs at the edge where the sample window is
    /// built, never in the algebra — clamping here would make the op
    /// non-associative, which is the same bug class as Zip's bracket list.
    #[test]
    fn gate_multiplies_and_nudge_adds_without_clamping() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

        Compose {
            operations: vec![
                Op(Gate {
                    m: Rational64::new(2, 1),
                }),
                Op(Gate {
                    m: Rational64::new(1, 2),
                }),
                Op(Nudge {
                    a: Rational64::new(1, 8),
                }),
                Op(Nudge {
                    a: Rational64::new(1, 8),
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let p = &input.operations[0][0];
        assert_eq!(p.gate, Rational64::new(1, 1), "2 * 1/2, not clamped to 1 then halved");
        assert_eq!(p.nudge, Rational64::new(1, 4));
    }

    /// Articulation never touches `l`, so it cannot move a voice's total.
    #[test]
    fn gate_does_not_change_length() {
        let mut gated = NormalForm::init();
        let mut pt = make_parse_table();
        Compose {
            operations: vec![
                Op(Sequence {
                    operations: vec![Op(AsIs), Op(AsIs), Op(AsIs)],
                }),
                Op(Gate {
                    m: Rational64::new(1, 3),
                }),
            ],
        }
        .apply_to_normal_form(&mut gated, &mut pt)
        .unwrap();

        assert_eq!(gated.length_ratio, Rational64::new(3, 1));
        for p in &gated.operations[0] {
            assert_eq!(p.l, Rational64::new(1, 1), "the SLOT is untouched");
            assert_eq!(p.gate, Rational64::new(1, 3), "only the window moved");
        }
    }

    /// `Gate 0` must read as silent. That predicate is what `silence_next`
    /// consults, and `silence_next` is what releases the PREVIOUS note — so
    /// a gate-0 note has to free its predecessor exactly as an `Fm 0` does,
    /// or the new op and the idiom it replaces differ in a way nobody would
    /// guess from either spelling.
    #[test]
    fn a_fully_closed_gate_counts_as_silence() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();
        Gate {
            m: Rational64::new(0, 1),
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        assert!(input.operations[0][0].is_silent());
    }

    /// THE BRACKET LIST IS THE RHYTHM — one step per element, cycled. It read
    /// the other way first (a list of patterns, all applied at once) and that
    /// MULTIPLIED them: `Zip [Lm 3, Lm 1]` gave every note l * 3 * 1, a
    /// uniform stretch with no alternation. It rendered and sounded nearly
    /// right, which is what made it worth a test of its own.
    #[test]
    fn the_bracket_list_is_one_rhythm_not_several_patterns() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

        Compose {
            operations: vec![
                Op(Sequence {
                    operations: vec![Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs)],
                }),
                Op(Zip {
                    operations: vec![
                        Op(Length {
                            m: Rational64::new(3, 1),
                        }),
                        Op(Length {
                            m: Rational64::new(1, 1),
                        }),
                    ],
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let lengths: Vec<Rational64> = input.operations[0].iter().map(|p| p.l).collect();
        assert_eq!(
            lengths,
            vec![
                Rational64::new(3, 1),
                Rational64::new(1, 1),
                Rational64::new(3, 1),
                Rational64::new(1, 1),
            ],
            "alternating, NOT four notes of 3 * 1 = 3"
        );
    }

    /// `Zip [a, b]` is `Zip [Seq [a, b]]` — the list is sugar for the
    /// sequence, so a named talea and an inline one are the same thing.
    #[test]
    fn an_inline_list_equals_a_named_sequence() {
        let steps = vec![
            Op(Length {
                m: Rational64::new(3, 1),
            }),
            Op(Length {
                m: Rational64::new(1, 2),
            }),
        ];
        let subject = Sequence {
            operations: vec![Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs)],
        };

        let mut inline = NormalForm::init();
        Compose {
            operations: vec![
                Op(subject.clone()),
                Op(Zip {
                    operations: steps.clone(),
                }),
            ],
        }
        .apply_to_normal_form(&mut inline, &mut make_parse_table())
        .unwrap();

        let mut named = NormalForm::init();
        Compose {
            operations: vec![
                Op(subject),
                Op(Zip {
                    operations: vec![Op(Sequence { operations: steps })],
                }),
            ],
        }
        .apply_to_normal_form(&mut named, &mut make_parse_table())
        .unwrap();

        assert_eq!(inline, named);
    }

    /// Several cycles at once is what CHAINING is for — each `Zip` is one
    /// talea, and they layer because a talea only multiplies fields into
    /// events that already exist.
    #[test]
    fn chained_zips_layer_independent_cycles() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

        Compose {
            operations: vec![
                Op(Sequence {
                    operations: vec![Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs)],
                }),
                // a 2-step rhythm
                Op(Zip {
                    operations: vec![
                        Op(Length {
                            m: Rational64::new(2, 1),
                        }),
                        Op(Length {
                            m: Rational64::new(1, 1),
                        }),
                    ],
                }),
                // under a 3-step dynamic
                Op(Zip {
                    operations: vec![
                        Op(Gain {
                            m: Rational64::new(1, 1),
                        }),
                        Op(Gain {
                            m: Rational64::new(1, 2),
                        }),
                        Op(Gain {
                            m: Rational64::new(1, 4),
                        }),
                    ],
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        let voice = &input.operations[0];
        assert_eq!(voice.len(), 6, "neither cycle changes the event count");
        let lengths: Vec<Rational64> = voice.iter().map(|p| p.l).collect();
        let gains: Vec<Rational64> = voice.iter().map(|p| p.g).collect();
        let two = Rational64::new(2, 1);
        let one = Rational64::new(1, 1);
        assert_eq!(lengths, vec![two, one, two, one, two, one], "period 2");
        assert_eq!(
            gains,
            vec![
                one,
                Rational64::new(1, 2),
                Rational64::new(1, 4),
                one,
                Rational64::new(1, 2),
                Rational64::new(1, 4)
            ],
            "period 3, running against the period-2 rhythm"
        );
    }

    /// THE DOUBLE-APPLICATION TRAP. Patterns normalize against a UNIT form,
    /// never against the input — otherwise the subject's own fm gets
    /// multiplied back into the result once per pattern, and a fifth becomes
    /// a ninth.
    #[test]
    fn a_pattern_does_not_receive_the_subject() {
        let mut input = NormalForm::init();
        let mut pt = make_parse_table();

        Compose {
            operations: vec![
                Op(TransposeM {
                    m: Rational64::new(3, 2),
                }),
                Op(Sequence {
                    operations: vec![Op(AsIs), Op(AsIs)],
                }),
                Op(Zip {
                    operations: vec![Op(Length {
                        m: Rational64::new(2, 1),
                    })],
                }),
            ],
        }
        .apply_to_normal_form(&mut input, &mut pt)
        .unwrap();

        for point in &input.operations[0] {
            assert_eq!(
                point.fm,
                Rational64::new(3, 2),
                "the subject's fm must appear ONCE, not once per pattern"
            );
        }
    }

    /// `get_length_ratio` cannot derive Zip's length from its patterns'
    /// ratios — it re-runs the zip. That makes it the arm most likely to
    /// drift away from what normalize actually produces, so pin them together.
    #[test]
    fn zip_length_ratio_agrees_with_normalize() {
        let zip = Compose {
            operations: vec![
                Op(Sequence {
                    operations: vec![Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs), Op(AsIs)],
                }),
                Op(Zip {
                    operations: vec![
                        Op(Length {
                            m: Rational64::new(3, 1),
                        }),
                        Op(Length {
                            m: Rational64::new(1, 2),
                        }),
                    ],
                }),
            ],
        };

        let mut pt = make_parse_table();
        let input = NormalForm::init();

        let mut normalized = input.clone();
        zip.apply_to_normal_form(&mut normalized, &mut pt).unwrap();

        let ratio = zip.get_length_ratio(&input, &mut pt).unwrap();

        assert_eq!(ratio * input.length_ratio, normalized.length_ratio);
        // 3, 1/2, 3, 1/2, 3
        assert_eq!(normalized.length_ratio, Rational64::new(10, 1));
    }
}
