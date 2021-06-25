use crate::ast::Op;
use crate::operations::{helpers::*, GetLengthRatio, NormalForm, Normalize};
use crate::Term;
use num_rational::{Ratio, Rational64};
use scop::Defs;
use weresocool_error::Error;

impl GetLengthRatio<Term> for Op {
    fn get_length_ratio(&self, defs: &mut Defs<Term>) -> Result<Rational64, Error> {
        match self {
            Op::AsIs {}
            | Op::AD { .. }
            | Op::Portamento { .. }
            | Op::Sine { .. }
            | Op::Triangle { .. }
            | Op::Square { .. }
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

            Op::Lambda {
                term,
                input_name: _,
            } => {
                unimplemented!();
                term.get_length_ratio(defs)
            }

            Op::FunctionCall { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;

                nf.get_length_ratio(defs)
            }

            Op::Id(id) => {
                let op = handle_id_error(id.to_string(), defs)?;
                op.get_length_ratio(defs)
            }

            Op::Length { m } | Op::Silence { m } => Ok(*m),

            Op::Sequence { operations } => {
                let mut new_total = Ratio::from_integer(0);
                for operation in operations {
                    new_total += operation.get_length_ratio(defs)?;
                }
                Ok(new_total)
            }

            Op::Compose { operations } => {
                let mut new_total = Ratio::from_integer(1);
                for operation in operations {
                    new_total *= operation.get_length_ratio(defs)?;
                }
                Ok(new_total)
            }

            Op::WithLengthRatioOf {
                with_length_of,
                main,
            } => {
                let main_length = match main {
                    Some(m) => m.get_length_ratio(defs)?,
                    None => Rational64::from_integer(1),
                };

                let target_length = with_length_of.get_length_ratio(defs)?;

                Ok(target_length / main_length)
            }

            Op::ModulateBy { .. } => Ok(Ratio::from_integer(1)),

            Op::Focus {
                main, op_to_apply, ..
            } => Ok(main.get_length_ratio(defs)? * op_to_apply.get_length_ratio(defs)?),

            Op::Overlay { operations } => {
                let mut max = Ratio::new(0, 1);
                for op in operations {
                    let next = op.get_length_ratio(defs)?;
                    if next > max {
                        max = next;
                    }
                }
                Ok(max)
            }
        }
    }
}
