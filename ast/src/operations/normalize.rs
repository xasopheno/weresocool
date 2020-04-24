use crate::operations::{
    helpers::*, substitute::get_fn_arg_map, GetLengthRatio, NormalForm, Normalize, Substitute,
};
use crate::{Defs, FunDef, Op, OscType, Term, Term::*};
use num_rational::Ratio;
use rand::prelude::*;
use weresocool_error::Error;

impl Normalize for Op {
    #[allow(clippy::cognitive_complexity)]
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) -> Result<(), Error> {
        match self {
            Op::AsIs => {}

            Op::Id(id) => {
                handle_id_error(id.to_string(), defs, None).apply_to_normal_form(input, defs)?;
            }
            //
            Op::FunctionCall { name, args } => {
                let f = handle_id_error(name.to_string(), defs, None);
                let arg_map = get_fn_arg_map(f.clone(), args);

                match f {
                    Term::FunDef(fun) => match fun {
                        FunDef { term, .. } => match *term {
                            Term::Op(op) => {
                                let result_op = op.substitute(input, defs, &arg_map)?;
                                result_op.apply_to_normal_form(input, defs)?
                            }
                            Term::Nf(_) => {
                                panic!("Function Op stored in NormalForm");
                            }
                            Term::FunDef(_) => {
                                panic!("Function Op stored in FunDef");
                            }
                            Term::Lop(lop) => {
                                let result = lop.substitute(input, defs, &arg_map)?;
                                result.apply_to_normal_form(input, defs)?
                            }
                        },
                    },
                    _ => {
                        panic!("Function stored in NormalForm");
                    }
                }
            }

            Op::Tag(name) => {
                let name = name.to_string();
                for seq in input.operations.iter_mut() {
                    for p_op in seq {
                        p_op.names.insert(name.clone());
                    }
                }
            }

            Op::FInvert => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        if *point_op.fm.numer() != 0 {
                            point_op.fm = point_op.fm.recip();
                        }
                    }
                }
            }

            Op::Reverse => {
                for voice in input.operations.iter_mut() {
                    voice.reverse();
                }
            }

            Op::Sine => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.osc_type = OscType::Sine
                    }
                }
            }

            Op::AD { attack, decay, asr } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.attack *= attack;
                        point_op.decay *= decay;
                        point_op.asr = *asr;
                    }
                }
            }

            Op::Portamento { m } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.portamento *= m;
                    }
                }
            }

            Op::Square => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.osc_type = OscType::Square
                    }
                }
            }

            Op::Noise => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.osc_type = OscType::Noise
                    }
                }
            }

            Op::TransposeM { m } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.fm *= m;
                    }
                }
            }

            Op::TransposeA { a } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.fa += a;
                    }
                }
            }

            Op::PanA { a } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.pa += a;
                    }
                }
            }

            Op::PanM { m } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.pm *= m;
                    }
                }
            }

            Op::Gain { m } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.g *= m;
                    }
                }
            }

            Op::Length { m } => {
                for voice in input.operations.iter_mut() {
                    for point_op in voice {
                        point_op.l *= m;
                    }
                }

                input.length_ratio *= m;
            }

            Op::Silence { m } => {
                for voice in input.operations.iter_mut() {
                    for mut point_op in voice {
                        point_op.fm = Ratio::new(0, 1);
                        point_op.fa = Ratio::new(0, 1);
                        point_op.g = Ratio::new(0, 1);
                        point_op.l *= m;
                    }
                }

                input.length_ratio *= m;
            }

            Op::Choice { operations } => {
                let mut rng = thread_rng();
                let choice = operations.choose(&mut rng).unwrap();
                choice.apply_to_normal_form(input, defs)?
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
                let target_length = with_length_of.get_length_ratio(defs);
                let main_length = main.get_length_ratio(defs);
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
                    pad_length(&mut nf, max_lr, defs);
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
