pub mod normalize {
    use operations::{NormalForm, Normalize, GetLengthRatio};
    use socool_parser::ast::{Op, Op::*};


    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: NormalForm) -> NormalForm {
            let mut output: NormalForm = vec![];
            match self {
                Op::AsIs => {
                    output = input
                }
                | Op::TransposeM { m } => {
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
                Op::Sequence { operations: _ } => { output = input },

                Op::Compose { operations } => {
                        for mut voice in input {
                            for compose_op in operations.clone() {
                            let mut new_voice = vec![];
                            for op in voice.clone() {
                                new_voice.push(
                                    Op::Compose {
                                        operations: vec![op, compose_op.clone()]
                                    }
                                )
                            }
                        }
                    }
                }
//
//                Op::WithLengthRatioOf {
//                    with_length_of: _,
//                    main: _,
//                } => None,

                Op::Overlay { operations: _ } => { output = input }
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

}


