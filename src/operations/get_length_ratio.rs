pub mod get_length_ratio {
    use num_rational::{Ratio, Rational64};
    use operations::NormalForm;
    use socool_parser::ast::Op;

    use operations::GetLengthRatio;

    extern crate num_rational;

    impl GetLengthRatio for Op {
        fn get_length_ratio(&self, input: &NormalForm) -> Rational64 {
            match self {
                Op::AsIs {}
                | Op::Sine {}
                | Op::Square {}
                | Op::Noise {}
                | Op::FInvert {}
                | Op::Reverse {}
                | Op::TransposeM { m: _ }
                | Op::TransposeA { a: _ }
                | Op::PanA { a: _ }
                | Op::PanM { m: _ }
                | Op::Gain { m: _ } => Ratio::from_integer(1),

                Op::Length { m } | Op::Silence { m } => *m,

                Op::Sequence { operations } => {
                    let mut new_total = Ratio::from_integer(0);
                    for operation in operations {
                        new_total += operation.get_length_ratio(input);
                    }
                    new_total
                }

                Op::Compose { operations } => {
                    let mut new_total = Ratio::from_integer(1);
                    for operation in operations {
                        new_total *= operation.get_length_ratio(input);
                    }
                    new_total
                }

                Op::Choice { operations } => operations[0].get_length_ratio(input),

                Op::WithLengthRatioOf {
                    with_length_of,
                    main: _,
                } => with_length_of.get_length_ratio(input),

                Op::ModulateBy { operations: _ } => input.get_nf_length_ratio(),

                Op::Overlay { operations } => {
                    let mut max = Ratio::new(0, 1);
                    for op in operations {
                        let next = op.get_length_ratio(input);
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
