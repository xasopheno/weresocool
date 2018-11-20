pub mod get_length_ratio {
    use operations::GetLengthRatio;
    use socool_parser::ast::Op;

    impl GetLengthRatio for Op {
        fn get_length_ratio(&self) -> f32 {
            match self {
                Op::AsIs {}
                | Op::Reverse {}
                | Op::TransposeM { m: _ }
                | Op::TransposeA { a: _ }
                | Op::PanA { a: _ }
                | Op::PanM { m: _ }
                | Op::Gain { m: _ }
                => 1.0,
//
                Op::Length { m } |
                Op::Silence { m } => *m,
//
                Op::Sequence { operations } => {
                    let mut new_total = 0.0;
                    for operation in operations {
                        new_total += operation.get_length_ratio();
                    }
                    new_total
                }

                Op::Compose { operations } => {
                    let mut new_total = 1.0;
                    for operation in operations {
                        new_total *= operation.get_length_ratio();
                    }
                    new_total
                }

                Op::WithLengthRatioOf {
                    with_length_of,
                    main: _,
                } => with_length_of.get_length_ratio(),

                Op::Overlay { operations } => {
                    let mut max = 0.0;
                    for op in operations {
                        let next = op.get_length_ratio();
                        if next > max {
                            max = next;
                        }
                    }
                    max
                }
            }
        }
    }
}
