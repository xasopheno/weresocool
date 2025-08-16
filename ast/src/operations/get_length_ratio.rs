use crate::ast::Op;
use crate::operations::{helpers::*, GetLengthRatio, NormalForm, Normalize, Defs};
use crate::Term;
use num_rational::{Ratio, Rational64};
use weresocool_error::Error;

impl GetLengthRatio for Op {
    fn get_length_ratio(
        &self,
        normal_form: &NormalForm,
        defs: &mut Defs,
    ) -> Result<Rational64, Error> {
        match self {
            Op::AsIs {}
            | Op::Color(_)
            | Op::Follow(..)
            | Op::Out {}
            | Op::Lowpass { .. }
            | Op::FMOsc { .. }
            | Op::Highpass { .. }
            | Op::Bandpass { .. }
            | Op::AD { .. }
            | Op::ASDR { .. }
            | Op::Portamento { .. }
            | Op::Sine { .. }
            | Op::Triangle { .. }
            | Op::Square { .. }
            | Op::Saw
            | Op::Noise {}
            | Op::FInvert {}
            | Op::Reverse {}
            | Op::Reverb { .. }
            | Op::TransposeM { .. }
            | Op::TransposeA { .. }
            | Op::PanA { .. }
            | Op::PanM { .. }
            | Op::Tag(_)
            | Op::WGSL(_)
            | Op::Gain { .. }
            | Op::Keep
            | Op::Midi { .. } => Ok(Ratio::from_integer(1)),

            Op::CSV1d { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;

                nf.get_length_ratio(normal_form, defs)
            }

            Op::CSV2d { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;

                nf.get_length_ratio(normal_form, defs)
            }

            Op::Lambda {
                term,
                input_name,
                scope,
            } => {
                if let Some(name) = input_name {
                    defs.ops.insert(scope, name, Term::Nf(normal_form.to_owned()));
                };
                term.get_length_ratio(normal_form, defs)
            }

            Op::FunctionCall { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;

                nf.get_length_ratio(normal_form, defs)
            }

            Op::Id(id) => {
                let op = handle_id_error(id.to_string(), defs)?;
                op.get_length_ratio(normal_form, defs)
            }

            Op::Length { m } | Op::Silence { m } => Ok(*m),

            Op::Sequence { operations } => {
                let mut new_total = Ratio::from_integer(0);
                for operation in operations {
                    new_total += operation.get_length_ratio(normal_form, defs)?;
                }
                Ok(new_total)
            }

            Op::Compose { operations } => {
                let mut new_total = Ratio::from_integer(1);
                for operation in operations {
                    new_total *= operation.get_length_ratio(normal_form, defs)?;
                }
                Ok(new_total)
            }

            Op::WithLengthRatioOf {
                with_length_of,
                main,
            } => {
                let main_length = match main {
                    Some(m) => m.get_length_ratio(normal_form, defs)?,
                    None => Rational64::from_integer(1),
                };

                let target_length = with_length_of.get_length_ratio(normal_form, defs)?;

                Ok(target_length / main_length)
            }

            Op::ModulateBy { .. } => Ok(Ratio::from_integer(1)),

            Op::Trim { operations } => {
                // Find Keep position and calculate length ratio for that segment
                let keep_position = operations
                    .iter()
                    .position(|op| matches!(op, Term::Op(Op::Keep)))
                    .unwrap_or(0); // Default to first segment if no Keep found

                // Calculate total length of all segments
                let mut total_length = Ratio::from_integer(0);
                let mut segment_length = Ratio::from_integer(0);
                
                for (i, op) in operations.iter().enumerate() {
                    let op_length = op.get_length_ratio(normal_form, defs)?;
                    if i == keep_position {
                        segment_length = op_length;
                    }
                    total_length += op_length;
                }

                // Return the proportion of the total that this segment represents
                if total_length == Ratio::from_integer(0) {
                    Ok(Ratio::from_integer(0))
                } else {
                    Ok(segment_length / total_length)
                }
            }

            Op::Focus {
                main, op_to_apply, ..
            } => Ok(main.get_length_ratio(normal_form, defs)?
                * op_to_apply.get_length_ratio(normal_form, defs)?),

            Op::Overlay { operations } => {
                let mut max = Ratio::new(0, 1);
                for op in operations {
                    let next = op.get_length_ratio(normal_form, defs)?;
                    if next > max {
                        max = next;
                    }
                }
                Ok(max)
            }
        }
    }
}
