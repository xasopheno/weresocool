pub mod get_length_ratio {
    use num_rational::{Ratio, Rational64};
    use socool_parser::ast::Op;

    use operations::GetGainRatio;

    extern crate num_rational;

    impl GetGainRatio for Op {
        fn get_gain_ratio(&self) -> Rational64 {
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
                | Op::Length { m: _ } => Ratio::from_integer(1),

                Op::Gain { m } => *m,

                Op::Silence { m: _ } => Ratio::from_integer(0),

                Op::Sequence { operations } => {
                    let mut max = Ratio::from_integer(0);
                    for operation in operations {
                        let gr = operation.get_gain_ratio();
                        if gr > max {
                            max = gr;
                        }
                    }

                    max
                }

                Op::Compose { operations } => {
                    let mut max = Ratio::from_integer(0);
                    for operation in operations {
                        let gr = operation.get_gain_ratio();
                        if gr > max {
                            max = gr;
                        }
                    }

                    max
                }

                Op::Choice { operations } => operations[0].get_gain_ratio(),

                Op::WithLengthRatioOf {
                    with_length_of,
                    main,
                } => main.get_gain_ratio(),

                Op::WithGainRatioOf {
                    with_gain_of,
                    main: _,
                } => with_gain_of.get_gain_ratio(),

                Op::ModulateBy { operations: _ } => Ratio::from_integer(1),

                Op::Overlay { operations } => {
                    let mut max = Ratio::new(0, 1);
                    for op in operations {
                        let next = op.get_gain_ratio();
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
