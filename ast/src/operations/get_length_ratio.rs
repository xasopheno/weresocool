use crate::ast::Op;
use crate::operations::{helpers::*, GetLengthRatio, NormalForm, Normalize, Substitute, Defs};
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
            | Op::Keeper(_)
            | Op::WGSL(_)
            | Op::Gain { .. }
            | Op::Midi { .. }
            | Op::Hue { .. }
            | Op::Saturation { .. }
            | Op::Brightness { .. }
            | Op::Vibrance { .. }
            | Op::Gamma { .. }
            | Op::ColorBlend { .. }
            | Op::ColorAdd { .. }
            | Op::ColorGradient { .. }
            | Op::ColorMix { .. }
            | Op::Wavefolder { .. }
            | Op::SoftClip { .. }
            | Op::Overdrive { .. }
            | Op::Bitcrusher { .. }
            | Op::Tanh { .. } => Ok(Ratio::from_integer(1)),

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

            Op::FromSound { .. } => {
                let mut nf = NormalForm::init();
                self.apply_to_normal_form(&mut nf, defs)?;

                nf.get_length_ratio(normal_form, defs)
            }

            Op::FromSoundYin { .. } => {
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

            Op::Length { m, .. } | Op::Silence { m } => Ok(*m),

            Op::Sequence { operations, .. } => {
                let mut new_total = Ratio::from_integer(0);
                for operation in operations {
                    new_total += operation.get_length_ratio(normal_form, defs)?;
                }
                Ok(new_total)
            }

            Op::Compose { operations, .. } => {
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

            Op::ModulateBy { operations, output } => {
                // Collect keeper names and their lengths
                let mut keeper_names: Vec<String> = vec![];
                let mut keeper_lengths: Vec<Rational64> = vec![];
                let mut total_modulator_length = Ratio::from_integer(0);

                for operation in operations {
                    let op_length = operation.get_length_ratio(normal_form, defs)?;
                    total_modulator_length += op_length;

                    let keeper_name = match operation {
                        Term::Op(Op::Keeper(name)) => Some(name.clone()),
                        Term::Op(Op::Compose { operations, .. }) if !operations.is_empty() => {
                            match &operations[0] {
                                Term::Op(Op::Keeper(name)) => Some(name.clone()),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(name) = keeper_name {
                        keeper_names.push(name);
                        keeper_lengths.push(op_length);
                    }
                }

                let has_keepers = !keeper_names.is_empty();

                if has_keepers {
                    // Scale to fit input (which has length 1 in this context)
                    let scale = Ratio::from_integer(1) / total_modulator_length;
                    let scaled_keeper_lengths: Vec<_> = keeper_lengths
                        .iter()
                        .map(|l| *l * scale)
                        .collect();

                    let keeper_total: Rational64 = scaled_keeper_lengths.iter().sum();

                    if let Some(output_ops) = output {
                        // Bind keeper names to their scaled lengths via substitute
                        let scope = defs.ops.create_uuid_scope();
                        for (i, name) in keeper_names.iter().enumerate() {
                            let length_op = Op::Length { m: scaled_keeper_lengths[i] };
                            defs.ops.insert(&scope, name, Term::Op(length_op));
                        }

                        // Substitute to resolve name references, then get length
                        let mut nf_clone = normal_form.clone();
                        let mut total = Ratio::from_integer(0);
                        for op in output_ops {
                            let substituted = op.substitute(&mut nf_clone, defs)?;
                            total += substituted.get_length_ratio(normal_form, defs)?;
                        }
                        Ok(total)
                    } else {
                        // Slice behavior - return sum of keeper lengths
                        Ok(keeper_total)
                    }
                } else {
                    // Normal ModulateBy - doesn't change length
                    Ok(Ratio::from_integer(1))
                }
            }

            Op::Focus {
                main, op_to_apply, ..
            } => Ok(main.get_length_ratio(normal_form, defs)?
                * op_to_apply.get_length_ratio(normal_form, defs)?),

            Op::Overlay { operations, .. } => {
                let mut max = Ratio::new(0, 1);
                for op in operations {
                    let next = op.get_length_ratio(normal_form, defs)?;
                    if next > max {
                        max = next;
                    }
                }
                Ok(max)
            }

            Op::Choose { operations } => {
                if operations.is_empty() {
                    return Err(Error::with_msg("Empty Choose!"));
                }
                // Use rand_ctx to select an index
                let n = std::num::NonZeroUsize::new(operations.len())
                    .ok_or_else(|| Error::with_msg("Choose with zero operations"))?;
                let idx = defs.rand_ctx.index(n, 0);
                // Get length of the selected operation
                operations[idx].get_length_ratio(normal_form, defs)
            }

            Op::Repeat { operations, count } => {
                // Calculate length of one iteration
                let mut total_ratio = Ratio::from_integer(1);
                for op in operations {
                    total_ratio *= op.get_length_ratio(normal_form, defs)?;
                }
                // Multiply by repeat count
                Ok(total_ratio * Ratio::from_integer(*count))
            }
        }
    }
}
