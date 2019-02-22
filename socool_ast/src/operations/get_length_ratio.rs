pub mod get_length_ratio {
    use crate::ast::{Op, OpOrNfTable};
    use crate::operations::{helpers::*, GetLengthRatio, NormalForm, Normalize};
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
                | Op::Fid(_)
                | Op::Gain { .. } => Ratio::from_integer(1),

                Op::FunctionDef { name, .. } => panic!(
                    "\
                     Trying to get LengthRatio of function called, {:?}\
                     Can't get LengthRatio of Function, Don't pass FunctionDef to FitLength",
                    name
                ),
                Op::FunctionCall { .. } => {
                    let mut nf = NormalForm::init();
                    self.apply_to_normal_form(&mut nf, table);

                    nf.get_length_ratio(table)
                }

                Op::Id(id) => {
                    let op = handle_id_error(id.to_string(), table);
                    op.get_length_ratio(table)
                }

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
                    main,
                } => {
                    let target_length = with_length_of.get_length_ratio(table);
                    let main_length = main.get_length_ratio(table);

                    target_length / main_length
                }

                Op::ModulateBy { operations: _ } => Ratio::from_integer(1),

                Op::Focus {
                    main, op_to_apply, ..
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
