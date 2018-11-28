pub mod normalize {
    extern crate num_rational;
    use num_rational::{Ratio, Rational};
    use operations::{GetLengthRatio, NormalForm, Normalize, PointOp};
    use socool_parser::ast::Op;
    use std::cmp::Ordering::{Equal, Greater, Less};

    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: &mut NormalForm) {
            match self {
                Op::AsIs => {}

                Op::Reverse => {
                    for mut voice in input.operations.iter_mut() {
                        voice.reverse();
                    }
                }

                Op::TransposeM { m } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.fm *= m;
                        }
                    }
                }

                Op::TransposeA { a } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.fa += a;
                        }
                    }
                }

                Op::PanA { a } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.pa += a;
                        }
                    }
                }

                Op::PanM { m } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.pm *= m;
                        }
                    }
                }

                Op::Gain { m } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.g *= m;
                        }
                    }
                }

                Op::Length { m } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.l *= m;
                        }
                    }

                    input.length_ratio *= m;
                }

                Op::Silence { m } => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            point_op.fm = Ratio::new(0, 1);
                            point_op.fa = Ratio::new(0, 1);
                            point_op.g = Ratio::new(0, 1);
                            point_op.l = point_op.l * m;
                        }
                    }

                    input.length_ratio = *m;

                }

                Op::Sequence { operations } => {
                    let mut result = NormalForm::init_empty();

                    for op in operations {
                        let mut input_clone = input.clone();
                        op.apply_to_normal_form(&mut input_clone);
                        result = join_sequence(result, input_clone);
                    }

                    *input = result
                }

                Op::Compose { operations } => {
                    for op in operations {
                        op.apply_to_normal_form(input);
                    }
                }

                Op::WithLengthRatioOf {
                    with_length_of,
                    main,
                } => {
                    let target_length = with_length_of.get_length_ratio();
                    let main_length = main.get_length_ratio();
                    let ratio = target_length / main_length;
                    let new_op = Op::Length { m: ratio };

                    new_op.apply_to_normal_form(input);

                    input.length_ratio = ratio;
                }

                Op::Overlay { operations } => {
                    let mut result = NormalForm::init_empty();
                    for op in operations {
                        let mut input_clone = input.clone();
                        op.apply_to_normal_form(&mut input_clone);
                        result.operations.append(&mut input_clone.operations);
                    }

                    match_length(input);
                    *input = result
                }
            }
        }
    }

    fn match_length(input: &mut NormalForm) {
        let max_len = get_max_length_ratio(&input);
        for voice in input.operations.iter_mut() {
            let mut voice_len = Ratio::new(0, 1);
            for point_op in voice.iter() {
                voice_len += point_op.get_length_ratio()
            }
            if voice_len < max_len {
                voice.push(PointOp {
                    fm: Ratio::new(0, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(0, 1),
                    l: max_len - voice_len,
                });
            }
        }

        input.length_ratio = max_len;
    }

    fn get_max_length_ratio(input: &NormalForm) -> Rational {
        let mut max_len = Ratio::new(0, 1);
        for voice in input.operations.iter() {
            let mut voice_len = Ratio::new(0, 1);
            for op in voice {
                voice_len += op.get_length_ratio()
            }

            if voice_len > max_len {
                max_len = voice_len
            }
        }

        max_len
    }

    fn join_sequence(mut l: NormalForm, mut r: NormalForm) -> NormalForm {
        if l.operations.len() == 0 {
            return r;
        }

        let diff = l.operations.len() as isize - r.operations.len() as isize;
        match diff.partial_cmp(&0).unwrap() {
            Equal => {}
            Greater => {
                for _ in 0..diff {
                    r.operations.push(vec![PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: r.length_ratio,
                    }])
                }
            }
            Less => {
                for _ in 0..-diff {
                    l.operations.push(vec![PointOp {
                        fm: Ratio::new(0, 1),
                        fa: Ratio::new(0, 1),
                        pm: Ratio::new(1, 1),
                        pa: Ratio::new(0, 1),
                        g: Ratio::new(0, 1),
                        l: l.length_ratio,
                    }])
                }
            }
        }

        let mut result = NormalForm::init_empty();
        for (left, right) in l.operations.iter_mut().zip(r.operations.iter_mut()) {
            left.append(right);

            result.operations.push(left.clone());
            result.length_ratio += result.length_ratio
        }

        result
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate num_rational;
    extern crate socool_parser;
    use num_rational::Ratio;
    use operations::{NormalForm, Normalize, PointOp};
    use socool_parser::ast::{Op, Op::*};

    #[test]
    fn normalize_asis() {
        let mut input = NormalForm::init();
        AsIs.apply_to_normal_form(&mut input);
        let expected = NormalForm {
            operations: vec![vec![PointOp::init()]],
            length_ratio: Ratio::new(1, 1),
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_tm() {
        let mut input = NormalForm::init();
        TransposeM {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(2, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_ta() {
        let mut input = NormalForm::init();
        TransposeA {
            a: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(2, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_pan_m() {
        let mut input = NormalForm::init();
        PanM {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(2, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_pan_a() {
        let mut input = NormalForm::init();
        PanA {
            a: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(2, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(1, 1),
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_gain() {
        let mut input = NormalForm::init();
        Gain {
            m: Ratio::new(2, 1),
        }
        .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(1, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(2, 1),
                l: Ratio::new(1, 1),
            }]],
        };

        assert_eq!(input, expected);
    }
    #[test]
    fn normalize_silence() {
        let mut input = NormalForm::init();
        Silence {
            m: Ratio::new(2, 1),
        }
            .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(0, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(0, 1),
                l: Ratio::new(2, 1),
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_length() {
        let mut input = NormalForm::init();
        Length {
            m: Ratio::new(2, 1),
        }
            .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![PointOp {
                fm: Ratio::new(1, 1),
                fa: Ratio::new(0, 1),
                pm: Ratio::new(1, 1),
                pa: Ratio::new(0, 1),
                g: Ratio::new(1, 1),
                l: Ratio::new(2, 1),
            }]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_compose() {
        let mut input = NormalForm::init();

        Compose {
            operations: vec![
                TransposeM { m: Ratio::new(2, 1), },
                Length { m: Ratio::new(2, 1), }
            ],
        }
            .apply_to_normal_form(&mut input);

        let expected = NormalForm {
            length_ratio: Ratio::new(2, 1),
            operations: vec![vec![
                PointOp {
                    fm: Ratio::new(2, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                },
            ]],
        };

        assert_eq!(input, expected);
    }

    #[test]
    fn normalize_sequence() {
        let mut input = NormalForm::init();

        Sequence {
            operations: vec![
                TransposeM { m: Ratio::new(2, 1), },
                Length { m: Ratio::new(2, 1), }
            ],
        }
            .apply_to_normal_form(&mut input);

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
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                },
            ]],
        };

        assert_eq!(input, expected);
    }
}
