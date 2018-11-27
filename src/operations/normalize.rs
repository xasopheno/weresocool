pub mod normalize {
    extern crate num_rational;
    use num_rational::{Ratio, Rational};
    use operations::{GetLengthRatio, NormalForm, Normalize, PointOp};
    use socool_parser::ast::{Op, Op::*};
    use std::cmp::Ordering::{Equal, Greater, Less};

    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: &mut NormalForm) {
            match self {
                Op::AsIs => {}

                Op::Reverse => {
                    for mut voice in input.iter_mut() {
                        voice.reverse();
                    }
                }

                Op::TransposeM { m } => {
                    for mut voice in input.iter_mut() {
                        for mut point_op in voice {
                            point_op.fm *= m;
                        }
                    }
                }

                Op::TransposeA { a } => {
                    for mut voice in input.iter_mut()  {
                        for mut point_op in voice {
                            point_op.fa += a;
                        }
                    }
                }

                Op::PanA { a } => {
                    for mut voice in input.iter_mut()  {
                        for mut point_op in voice {
                            point_op.pa += a;
                        }
                    }
                }

                Op::PanM { m } => {
                    for mut voice in input.iter_mut()  {
                        for mut point_op in voice {
                            point_op.pm *= m;
                        }
                    }
                }

                Op::Gain { m } => {
                    for mut voice in input.iter_mut()  {
                        for mut point_op in voice {
                            point_op.g *= m;
                        }
                    }
                }

                Op::Length { m } => {
                    for mut voice in input.iter_mut()  {
                        for mut point_op in voice {
                            point_op.l *= m;
                        }
                    }
                }

                Op::Silence { m } => {
                    for mut voice in input.iter_mut()  {
                        for mut point_op in voice {
                            point_op.fm = Ratio::new(0,1);
                            point_op.fa = Ratio::new(0,1);
                            point_op.g = Ratio::new(0,1);
                            point_op.l = point_op.l * m;
                        }
                    }
                }

                Op::Sequence { operations } => {
                    let mut result = vec![];

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
                }

                Op::Overlay { operations } => {
                    let mut result = vec![];
                    for op in operations {
                        let mut input_clone = input.clone();
                        op.apply_to_normal_form(&mut input_clone);
                        result.append(&mut input_clone);
                    }

                    match_length(input);

                    *input = result
                }
            }

        }
    }

    fn match_length(input: &mut NormalForm) {
        let max_len = get_max_length_ratio(&input);
        for voice in input.iter_mut() {
            let mut voice_len = Ratio::new(0, 1);
            for op in voice.iter() {
                voice_len += op.get_length_ratio()
            }
            if voice_len < max_len {
                voice.push(PointOp {
                    fm: Ratio::new(0,1),
                    fa: Ratio::new(0,1),
                    pm: Ratio::new(1,1),
                    pa: Ratio::new(0,1),
                    g: Ratio::new(0,1),
                    l: max_len - voice_len,
                });
            }
        }
    }

    fn get_max_length_ratio(input: &NormalForm) -> Rational {
        let mut max_len = Ratio::new(0, 1);
        for voice in input {
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
        if l.len() == 0 {
            return r;
        }

        let diff = l.len() as isize - r.len() as isize;
        match diff.partial_cmp(&0).unwrap() {
            Equal => {}
            Greater => {
                let r_max_len = get_max_length_ratio(&r);

                for _ in 0..diff {
                    r.push(vec![PointOp {
                        fm: Ratio::new(0,1),
                        fa: Ratio::new(0,1),
                        pm: Ratio::new(1,1),
                        pa: Ratio::new(0,1),
                        g: Ratio::new(0,1),
                        l: r_max_len,
                    }])
                }
            }
            Less => {
                let l_max_len = get_max_length_ratio(&l);

                for _ in 0..-diff {
                    l.push(vec![PointOp {
                        fm: Ratio::new(0,1),
                        fa: Ratio::new(0,1),
                        pm: Ratio::new(1,1),
                        pa: Ratio::new(0,1),
                        g: Ratio::new(0,1),
                        l: l_max_len,
                    }])
                }
            }
        }

        let mut result = vec![];
        for (left, right) in l.iter_mut().zip(r.iter_mut()) {
            left.append(right);

            result.push(left.clone())
        }

        result
    }
}
