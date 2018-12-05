pub mod normalize {
    extern crate num_rational;
    extern crate rand;
    use num_rational::{Ratio, Rational64};
    use operations::{GetLengthRatio, NormalForm, Normalize, PointOp};
    use socool_parser::ast::Op;
    use std::cmp::Ordering::{Equal, Greater, Less};
    use rand::prelude::*;


    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: &mut NormalForm) {
            match self {
                Op::AsIs => {}

                Op::FInvert => {
                    for mut voice in input.operations.iter_mut() {
                        for mut point_op in voice {
                            if *point_op.fm.numer() != 0 {
                                point_op.fm = point_op.fm.recip();
                            }
                        }
                    }
                }

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
                            point_op.fm = Ratio::new(0, 1);
                            point_op.fa = Ratio::new(0, 1);
                            point_op.g = Ratio::new(0, 1);
                            point_op.l = point_op.l * m;
                        }
                    }

                    input.length_ratio = *m;
                }

                Op::Choice { operations } => {
                    let mut choice = rand::thread_rng().choose(&operations).unwrap();
                    choice.apply_to_normal_form(input)
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

                    input.length_ratio = target_length;
                }

                Op::Overlay { operations } => {
                    let normal_forms: Vec<NormalForm> = operations
                        .iter()
                        .map(|op| {
                            let mut input_clone = input.clone();
                            op.apply_to_normal_form(&mut input_clone);
                            input_clone
                        })
                        .collect();

                    let max_lr = normal_forms
                        .iter()
                        .map(|nf| nf.length_ratio)
                        .max()
                        .expect("Normalize, max_lr not working");

                    let mut result = vec![];

                    for mut nf in normal_forms {
                        pad_length(&mut nf, max_lr);
                        result.append(&mut nf.operations);
                    }

                    *input = NormalForm {
                        operations: result,
                        length_ratio: max_lr,
                    };
                }
            }
        }
    }

    fn pad_length(input: &mut NormalForm, max_len: Rational64) {
        let input_lr = input.get_length_ratio();
        if input_lr < max_len {
            for voice in input.operations.iter_mut() {
                voice.push(PointOp {
                    fm: Ratio::new(0, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(0, 1),
                    l: max_len - input_lr,
                });
            }
        }
        input.length_ratio = max_len;
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
        }

        result.length_ratio += r.length_ratio;
        result.length_ratio += l.length_ratio;

        result
    }
}
