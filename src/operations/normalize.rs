pub mod normalize {
    use operations::{GetOperations, NormalForm, Normalize, GetLengthRatio};
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
//                Op::Length { m: _ } | Op::Silence { m: _ } => None,
//
                Op::Sequence { operations: _ }
                | Op::Compose { operations: _ }
                => { output = input },
//
//                Op::WithLengthRatioOf {
//                    with_length_of: _,
//                    main: _,
//                } => None,

                Op::Overlay { operations: _ } => { output = input }
            }

            output
        }
    }
}
