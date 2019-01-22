pub mod get_length_ratio {
    use num_rational::{Ratio, Rational64};
    use socool_parser::ast::Op;

    use operations::GetLengthRatio;

    extern crate num_rational;

    impl GetLengthRatio for Op {
        fn get_length_ratio(&self) -> Rational64 {
            match self {
                Op::AsIs {}
                | Op::Sine {}
                | Op::Square {}
                | Op::Noise {}
                | Op::FInvert {}
                | Op::Reverse {}
                | Op::TransposeM { .. }
                | Op::TransposeA { .. }
                | Op::PanA { .. }
                | Op::PanM { .. }
                | Op::Gain { .. } => Ratio::from_integer(1),

                Op::Length { m } | Op::Silence { m } => *m,

                Op::Sequence { operations } => {
                    let mut new_total = Ratio::from_integer(0);
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

                Op::Choice { operations } => operations[0].get_length_ratio(),

                Op::WithLengthRatioOf {
                    with_length_of,
                    main: _,
                } => with_length_of.get_length_ratio(),

                Op::ModulateBy { operations: _ } => Ratio::from_integer(1),

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
