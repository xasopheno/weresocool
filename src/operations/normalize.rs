pub mod normalize {
    extern crate num_rational;
    use num_rational::{Ratio, Rational};
    use operations::{GetLengthRatio, NormalForm, Normalize};
    use socool_parser::ast::{Op, Op::*};
    use std::cmp::Ordering::{Equal, Greater, Less};

    fn fmap_point_op(new_op: Op, input: &mut NormalForm) {
        for mut voice in input {
            for mut op in voice {
                *op = Op::Compose {
                    operations: vec![op.clone(), new_op.clone()],
                };
            }
        }
    }

    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: &mut NormalForm) {
            //            println!("{:?}", input);
            match self {
                Op::AsIs => {}

                Op::Reverse => {
                    for mut voice in input.iter_mut() {
                        voice.reverse();
                    }
                }

                Op::TransposeM { m } => {
                    fmap_point_op(Op::TransposeM { m: *m }, input);
                }

                Op::TransposeA { a } => {
                    fmap_point_op(Op::TransposeA { a: *a }, input);
                }

                Op::PanA { a } => {
                    fmap_point_op(Op::PanA { a: *a }, input);
                }

                Op::PanM { m } => {
                    fmap_point_op(Op::PanM { m: *m }, input);
                }

                Op::Gain { m } => {
                    fmap_point_op(Op::Gain { m: *m }, input);
                }

                Op::Length { m } => {
                    fmap_point_op(Op::Length { m: *m }, input);
                }

                Op::Silence { m } => {
                    let max_len = get_max_length_ratio(&input);

                    for voice in input.iter_mut() {
                        *voice = vec![Op::Silence { m: max_len * m }]
                    }
                }
                //
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
                    println!("{:?}", ratio);
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

                    *input = result
                }
            }

            match_length(input);
        }
    }

    fn match_length(input: &mut NormalForm) {
        let max_len = get_max_length_ratio(&input);
        for voice in input {
            let mut voice_len = Ratio::new(0, 1);
            for op in voice.clone() {
                voice_len += op.get_length_ratio()
            }
            if voice_len < max_len {
                voice.push(Silence {
                    m: max_len - voice_len,
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
        let l_max_len = get_max_length_ratio(&l);
        let r_max_len = get_max_length_ratio(&r);
        match diff.partial_cmp(&0).unwrap() {
            Equal => {}
            Greater => {
                for _ in 0..(diff.abs()) {
                    r.push(vec![Op::Silence { m: r_max_len }])
                }
            }
            Less => {
                for _ in 0..diff.abs() {
                    l.push(vec![Op::Silence { m: l_max_len }])
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
