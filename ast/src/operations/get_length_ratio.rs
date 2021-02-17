use crate::ast::{Defs, Op};
use crate::operations::{helpers::*, GetLengthRatio, NormalForm, Normalize};
use num_rational::{Ratio, Rational64};
use weresocool_error::Error;

impl GetLengthRatio for Op {
    fn get_length_ratio(&self, input: &NormalForm, defs: &Defs) -> Result<Rational64, Error> {
        match self {
            Op::AsIs {}
            | Op::Sine { .. }
            | Op::AD { .. }
            | Op::Portamento { .. }
            | Op::Square {}
            | Op::Noise {}
            | Op::FInvert {}
            | Op::Reverse {}
            | Op::Reverb { .. }
            | Op::TransposeM { .. }
            | Op::TransposeA { .. }
            | Op::PanA { .. }
            | Op::PanM { .. }
            | Op::Tag(_)
            | Op::Gain { .. } => Ok(Ratio::from_integer(1)),

            Op::FunctionCall { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;

                nf.get_length_ratio(input, defs)
            }

            Op::Id(id) => {
                let op = handle_id_error(id.to_string(), defs, None)?;
                op.get_length_ratio(input, defs)
            }

            Op::Length { m } | Op::Silence { m } => Ok(*m),

            Op::Sequence { operations } => {
                let mut new_total = Ratio::from_integer(0);
                for operation in operations {
                    new_total += operation.get_length_ratio(input, defs)?;
                }
                Ok(new_total)
            }

            Op::Compose { operations } => {
                let mut new_total = Ratio::from_integer(1);
                for operation in operations {
                    new_total *= operation.get_length_ratio(input, defs)?;
                }
                Ok(new_total)
            }

            Op::WithLengthRatioOf { with_length_of } => {
                let target_length = with_length_of.get_length_ratio(input, defs)?;
                let main_length = input.get_length_ratio(input, defs)?;
                dbg!(target_length / main_length);
                Ok(target_length)
            }

            Op::ModulateBy { .. } => Ok(Ratio::from_integer(1)),

            Op::Focus {
                main, op_to_apply, ..
            } => {
                Ok(main.get_length_ratio(input, defs)?
                    * op_to_apply.get_length_ratio(input, defs)?)
            }

            Op::Overlay { operations } => {
                let mut max = Ratio::new(0, 1);
                for op in operations {
                    let next = op.get_length_ratio(input, defs)?;
                    if next > max {
                        max = next;
                    }
                }
                Ok(max)
            }
        }
    }
}
