use crate::datagen::csv_to_normalform;
use crate::operations::Rational64;
use crate::operations::{
    helpers::*, substitute::insert_function_args, GetLengthRatio, NormalForm, Normalize, Substitute,
};
use crate::{FunDef, Op, OscType, Term, Term::*};
use num_rational::Ratio;
use num_traits::CheckedMul;
use scop::Defs;
use weresocool_error::Error;
use weresocool_shared::lossy_rational_mul;

impl Normalize<Term> for Op {
    #[allow(clippy::cognitive_complexity)]
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        defs: &mut Defs<Term>,
    ) -> Result<(), Error> {
        match self {
            Op::AsIs => {}
            Op::Lambda {
                term,
                input_name,
                scope,
            } => {
                if let Some(name) = input_name {
                    defs.insert(scope, name, Term::Nf(input.to_owned()));
                }
                let mut nf = NormalForm::init();
                term.apply_to_normal_form(&mut nf, defs)?;
                *input = nf;
            }

            Op::Id(id) => {
                handle_id_error(id, defs)?.apply_to_normal_form(input, defs)?;
            }

            Op::CSV { path, scale } => {
                csv_to_normalform(path, *scale)?.apply_to_normal_form(input, defs)?;
            }

            Op::FunctionCall { name, args } => {
                let f = handle_id_error(name.to_string(), defs)?;
                insert_function_args(&f, args, defs)?;

                match f {
                    Term::FunDef(fun) => {
                        let FunDef { term, .. } = fun;
                        match *term {
                            Term::Op(op) => {
                                let result_op = op.substitute(input, defs)?;
                                result_op.apply_to_normal_form(input, defs)?
                            }
                            Term::Nf(_) => {
                                println!("Function Op stored in NormalForm");
                                return Err(Error::with_msg("Function Op stored in NormalForm"));
                            }
                            Term::FunDef(_) => {
                                println!("Function Op stored in FunDef");
                                return Err(Error::with_msg("Function Op stored in FunDef"));
                            }
                            Term::Lop(lop) => {
                                let result = lop.substitute(input, defs)?;
                                result.apply_to_normal_form(input, defs)?
                            }
                            Term::Gen(gen_op) => {
                                let result = gen_op.substitute(input, defs)?;
                                result.apply_to_normal_form(input, defs)?
                            }
                        }
                    }
                    _ => {
                        println!("FunctionCall does not point to FunctionDef");
                        return Err(Error::with_msg(
                            "FunctionCall does not point to FunctionDef",
                        ));
                    }
                }
            }

            Op::Tag(name) => {
                let name = name.to_string();
                input.fmap_mut(|op| {
                    op.names.insert(name.clone());
                })
            }

            Op::FInvert => input.fmap_mut(|op| {
                if *op.fm.numer() != 0 {
                    op.fm = op.fm.recip();
                }
            }),

            Op::Reverse => {
                for voice in input.operations.iter_mut() {
                    voice.reverse();
                }
            }

            Op::Reverb { m } => input.fmap_mut(|op| {
                if m.is_some() {
                    op.reverb = *m
                }
            }),

            Op::AD { attack, decay, asr } => input.fmap_mut(|op| {
                op.attack *= attack;
                op.decay *= decay;
                op.asr = *asr;
            }),

            Op::Portamento { m } => input.fmap_mut(|op| {
                op.portamento *= m;
            }),

            Op::Sine { pow } => input.fmap_mut(|op| op.osc_type = OscType::Sine { pow: *pow }),

            Op::Triangle { pow } => {
                input.fmap_mut(|op| op.osc_type = OscType::Triangle { pow: *pow })
            }

            Op::Square { width } => {
                input.fmap_mut(|op| op.osc_type = OscType::Square { width: *width })
            }

            Op::Noise => input.fmap_mut(|op| op.osc_type = OscType::Noise),

            Op::TransposeM { m } => input.fmap_mut(|op| {
                op.fm = op
                    .fm
                    .checked_mul(m)
                    .unwrap_or_else(|| lossy_rational_mul(op.fm, *m))
            }),

            Op::TransposeA { a } => input.fmap_mut(|op| {
                op.fa += a;
            }),

            Op::PanA { a } => input.fmap_mut(|op| {
                op.pa += a;
            }),

            Op::PanM { m } => input.fmap_mut(|op| {
                op.pm = op
                    .pm
                    .checked_mul(m)
                    .unwrap_or_else(|| lossy_rational_mul(op.pm, *m))
            }),

            Op::Gain { m } => input.fmap_mut(|op| {
                op.g =
                    op.g.checked_mul(m)
                        .unwrap_or_else(|| lossy_rational_mul(op.g, *m))
            }),

            Op::Length { m } => {
                input.fmap_mut(|op| {
                    op.l =
                        op.l.checked_mul(m)
                            .unwrap_or_else(|| lossy_rational_mul(op.l, *m))
                });

                input.length_ratio *= m;
            }

            Op::Silence { m } => {
                input.fmap_mut(|op| {
                    op.fm = Ratio::new(0, 1);
                    op.fa = Ratio::new(0, 1);
                    op.g = Ratio::new(0, 1);
                    op.l *= m;
                });

                input.length_ratio *= m;
            }

            Op::Sequence { operations } => {
                let mut result = NormalForm::init_empty();
                for op in operations {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs)?;
                    result = join_sequence(result, input_clone);
                }

                *input = result
            }

            Op::Compose { operations } => {
                for op in operations {
                    op.apply_to_normal_form(input, defs)?;
                }
            }

            Op::WithLengthRatioOf {
                with_length_of,
                main,
            } => {
                let main_length = match main {
                    Some(m) => m.get_length_ratio(input, defs)?,
                    None => Rational64::from_integer(1),
                };
                let target_length = with_length_of.get_length_ratio(input, defs)?;
                let ratio = target_length / main_length;
                let new_op = Op::Length { m: ratio };

                new_op.apply_to_normal_form(input, defs)?;

                input.length_ratio = target_length;
            }

            Op::Focus {
                name,
                main,
                op_to_apply,
            } => {
                main.apply_to_normal_form(input, defs)?;
                let (named, rest) = input.clone().partition(name.to_string());

                let mut nf = NormalForm::init();
                op_to_apply.apply_to_normal_form(&mut nf, defs)?;
                let named_applied = nf * named;

                let mut result = NormalForm::init();

                Op::Overlay {
                    operations: vec![Nf(rest), Nf(named_applied)],
                }
                .apply_to_normal_form(&mut result, defs)?;

                *input = result
            }

            Op::ModulateBy { operations } => {
                let mut modulator = NormalForm::init_empty();

                for op in operations {
                    let mut nf = NormalForm::init();
                    op.apply_to_normal_form(&mut nf, defs)?;
                    modulator = join_sequence(modulator, nf);
                }

                Op::Length {
                    m: input.length_ratio / modulator.length_ratio,
                }
                .apply_to_normal_form(&mut modulator, defs)?;

                let mut result = NormalForm::init_empty();

                for modulation_line in modulator.operations.iter() {
                    for input_line in input.operations.iter() {
                        result
                            .operations
                            .push(modulate(input_line, modulation_line));
                    }
                }

                result.length_ratio = input.length_ratio;
                *input = result
            }

            Op::Overlay { operations } => {
                let normal_forms = operations
                    .iter()
                    .map(|op| {
                        let mut input_clone = input.clone();
                        op.apply_to_normal_form(&mut input_clone, defs)?;
                        Ok(input_clone)
                    })
                    .collect::<Result<Vec<NormalForm>, Error>>()?;

                let max_lr = normal_forms
                    .iter()
                    .map(|nf| nf.length_ratio)
                    .max()
                    .expect("Normalize, max_lr not working");

                let mut result = vec![];

                for mut nf in normal_forms {
                    pad_length(&mut nf, max_lr, defs)?;
                    result.append(&mut nf.operations);
                }

                *input = NormalForm {
                    operations: result,
                    length_ratio: max_lr,
                };
            }
        }
        Ok(())
    }
}
