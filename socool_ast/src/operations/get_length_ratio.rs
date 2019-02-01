pub mod get_length_ratio {
    use crate::ast::{Op, OpOrNfTable};
    use crate::operations::{helpers::*, GetLengthRatio};
    use num_rational::{Ratio, Rational64};

    extern crate num_rational;

    impl GetLengthRatio for Op {
        fn get_length_ratio(&self, table: &OpOrNfTable) -> Rational64 {
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
                | Op::Tag(_)
                | Op::Gain { .. } => Ratio::from_integer(1),

                Op::Id(id_vec) => handle_id_error(id_vec.to_vec(), table).get_length_ratio(table),

                Op::Length { m } | Op::Silence { m } => *m,

                Op::Sequence { operations } => {
                    let mut new_total = Ratio::from_integer(0);
                    for operation in operations {
                        new_total += operation.get_length_ratio(table);
                    }
                    new_total
                }

                Op::Compose { operations } => {
                    let mut new_total = Ratio::from_integer(1);
                    for operation in operations {
                        new_total *= operation.get_length_ratio(table);
                    }
                    new_total
                }

                Op::Choice { operations } => operations[0].get_length_ratio(table),

                Op::WithLengthRatioOf {
                    with_length_of,
                    main: _,
                } => with_length_of.get_length_ratio(table),

                Op::ModulateBy { operations: _ } => Ratio::from_integer(1),

                Op::Focus {
                    name: _,
                    main,
                    op_to_apply,
                } => main.get_length_ratio(table) * op_to_apply.get_length_ratio(table),

                Op::Overlay { operations } => {
                    let mut max = Ratio::new(0, 1);
                    for op in operations {
                        let next = op.get_length_ratio(table);
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
