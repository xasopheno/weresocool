pub mod normalize {
    use operations::{NormalForm, Normalize, GetLengthRatio};
    use socool_parser::ast::{Op, Op::*};
    use std::cmp::Ordering::{Less, Greater, Equal};

    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: NormalForm) -> NormalForm {
            let mut output: NormalForm = vec![];
            match self {
                Op::AsIs => {
                    output = input
                }

                Op::TransposeM { m } => {
                    for mut voice in input {
                        let mut new_voice = vec![];
                        for op in voice {
                            new_voice.push(
                                Op::Compose {
                                    operations: vec![op, Op::TransposeM { m: *m }]
                                }
                            )
                        }
                    }
                }
//                | Op::TransposeA { a: _ }
//                | Op::PanA { a: _ }
//                | Op::PanM { m: _ }
//                | Op::Gain { m: _ }
//
//                Op::Length { m: _ } |

                Op::Silence { m } => {
                    let max_len = get_max_length_ratio(&input);

                    for _i in 0..input.len() {
                        output.push(vec![Op::Silence { m: max_len * m }])
                    }
                },
//
                Op::Sequence { operations } => {
                    println!("in Sequence {:?}", operations);
                    let mut result = input.clone();

                    for op in operations {
                        result = join_sequence(result.clone(), op.apply_to_normal_form(input.clone()));
                        println!("{:?}", result);
                    }

                    output = result
                },

                Op::Compose { operations } => {
                    let mut result = vec![];
                    for op in operations {
                        result.push(op.apply_to_normal_form(input.clone()));
                    }

                    println!("in Compose {:?}", output);
                    output = result[0].clone()
                }
//
//                Op::WithLengthRatioOf {
//                    with_length_of: _,
//                    main: _,
//                } => None,

                Op::Overlay { operations } => {
                    println!("in Overlay {:?}", operations);
                    let mut voices = vec![];
                    for op in operations {
                        let result = op.apply_to_normal_form(input.clone());
                        println!("{:?}", result);
                        if result.len() > 0 {
                            voices.push(result[0].clone());
                        }

                    }
                    println!("End of Overlay {:?}", voices);
                    output = voices
                }
            }

            match_length(&mut output);

            output
        }
    }


    fn match_length(input: &mut NormalForm) {
        let max_len = get_max_length_ratio(&input);
        for voice in input {
            let mut voice_len = 0.0;
            for op in voice.clone() {
                voice_len += op.get_length_ratio()
            }
            if voice_len < max_len {
                voice.push(Silence {m: max_len - voice_len})
            }
        }
    }

    fn get_max_length_ratio(input: &NormalForm) -> f32 {
        let mut max_len = 0.0;
        for voice in input {
            let mut voice_len = 0.0;
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
        let diff = l.len() - r.len();
        let l_max_len = get_max_length_ratio(&l);
        let r_max_len = get_max_length_ratio(&r);
        match diff.partial_cmp(&0).unwrap() {
            Equal => {},
            Greater => {
                for i in 0..diff {
                    r.push(vec![Op::Silence {m: r_max_len}])
                }
            }
            Less => {
                for i in 0..diff {
                    l.push(vec![Op::Silence {m: l_max_len}])
                }
            }
        }
        for (l_voice, mut r_voice) in l.iter_mut().zip(r.iter_mut()) {
            l_voice.append(&mut r_voice)
        }

        l
    }

}


