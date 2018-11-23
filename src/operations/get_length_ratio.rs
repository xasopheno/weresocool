pub mod get_length_ratio {
    use operations::GetLengthRatio;
    use socool_parser::ast::Op;
    extern crate num_rational;
    use num_rational::{Rational, Ratio};

    impl GetLengthRatio for Op {
        fn get_length_ratio(&self) -> Rational {
            match self {
                Op::AsIs {}
                | Op::Reverse {}
                | Op::TransposeM { m: _ }
                | Op::TransposeA { a: _ }
                | Op::PanA { a: _ }
                | Op::PanM { m: _ }
                | Op::Gain { m: _ } => Ratio::from_integer(1),
                //
                Op::Length { m } | Op::Silence { m } => *m,
                //
                Op::Sequence { operations } => {
                    let mut new_total = Ratio::from_integer(1);
                    for operation in operations {
                        new_total += operation.get_length_ratio();
                    }
                    new_total
                }

                Op::Compose { operations } => {
                    let mut new_total = Ratio::from_integer(1);
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
                    let mut max = Ratio::new(0, 1);
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
