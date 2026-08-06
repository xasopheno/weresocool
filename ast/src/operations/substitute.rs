use crate::operations::{helpers::handle_id_error, NormalForm, Normalize, Substitute, Defs};
use crate::{FunDef, Op, Term};
use weresocool_error::Error;

pub fn insert_function_args(f: &Term, args: &[Term], defs: &mut Defs) -> Result<(), Error> {
    match f {
        Term::FunDef(fun) => {
            let FunDef { name, vars, .. } = fun;

            // Arity check: ensure argument count matches parameter count
            if vars.len() != args.len() {
                return Err(Error::with_msg(format!(
                    "Function '{}' expects {} argument(s) ({}), got {}",
                    name,
                    vars.len(),
                    vars.join(", "),
                    args.len()
                )));
            }

            let new_scope = defs.ops.create_uuid_scope();
            for (var, arg) in vars.iter().zip(args.iter()) {
                defs.ops.insert(&new_scope, var.to_string(), arg.clone());
            }
        }
        _ => {
            println!("FunctionCall does not point to FunctionDef");
            return Err(Error::with_msg(
                "FunctionCall does not point to FunctionDef",
            ));
        }
    }

    Ok(())
}

impl Substitute for Op {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<Term, Error> {
        match self {
            Op::Id(id) => handle_id_error(id, defs),

            Op::WithLengthRatioOf {
                main,
                with_length_of,
            } => {
                let with_length_of = with_length_of.substitute(normal_form, defs)?;
                let main = match main.as_ref() {
                    Some(m) => m.substitute(normal_form, defs)?,
                    None => Term::Nf(NormalForm::init()),
                };

                Ok(Term::Op(Op::WithLengthRatioOf {
                    main: Some(Box::new(main)),
                    with_length_of: Box::new(with_length_of),
                }))
            }

            Op::Focus {
                name,
                main,
                op_to_apply,
            } => {
                let mut nf = NormalForm::init();
                let m = main.substitute(normal_form, defs)?;
                m.apply_to_normal_form(&mut nf, defs)?;
                let (named, rest) = nf.partition(name.to_string());

                let op_to_apply = op_to_apply.substitute(normal_form, defs)?;

                let mut nf = NormalForm::init();
                op_to_apply.apply_to_normal_form(&mut nf, defs)?;
                let named_applied = nf * named;

                let mut result = NormalForm::init();

                Op::Overlay {
                    operations: vec![Term::Nf(named_applied), Term::Nf(rest)],
                }
                .apply_to_normal_form(&mut result, defs)?;

                Ok(Term::Nf(result))
            }
            Op::FunctionCall { name, args } => Ok(Term::Op(Op::FunctionCall {
                name: name.to_string(),
                args: substitute_operations(args.to_vec(), normal_form, defs)?,
            })),
            Op::Sequence { operations } => Ok(Term::Op(Op::Sequence {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
            })),
            Op::Overlay { operations } => Ok(Term::Op(Op::Overlay {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
            })),
            Op::Zip { operations } => Ok(Term::Op(Op::Zip {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
            })),
            Op::Compose { operations } => Ok(Term::Op(Op::Compose {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
            })),
            Op::ModulateBy { operations, output } => Ok(Term::Op(Op::ModulateBy {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
                output: match output {
                    Some(ops) => Some(substitute_operations(ops.to_vec(), normal_form, defs)?),
                    None => None,
                },
            })),
            Op::Choose { operations } => {
                // Select one operation using rand_ctx
                if operations.is_empty() {
                    return Err(Error::with_msg("Empty Choose!"));
                }
                let n = std::num::NonZeroUsize::new(operations.len())
                    .ok_or_else(|| Error::with_msg("Choose with zero operations"))?;
                let idx = defs.rand_ctx.index(n, 0);
                // Substitute only the selected operation
                operations[idx].substitute(normal_form, defs)
            }
            Op::Repeat { operations, count } => Ok(Term::Op(Op::Repeat {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
                count: *count,
            })),
            Op::Lambda {
                term,
                input_name,
                scope,
            } => {
                if let Some(name) = input_name {
                    defs.ops.insert(scope, name, Term::Nf(normal_form.to_owned()));
                }
                Ok(Term::Op(Op::Lambda {
                    input_name: input_name.to_owned(),
                    term: Box::new(term.substitute(normal_form, defs)?),
                    scope: scope.into(),
                }))
            }
            // LEAVES — ops with no operands to descend into, so substituting
            // one is substituting nothing. Listed rather than caught by a
            // `_` arm ON PURPOSE: this match is the only place a container
            // op's children get resolved, and a `_` here does not fail to
            // build when a new container forgets its arm — it silently stops
            // substituting that op's operands, so the op works everywhere
            // except inside a lambda or a function call. Exhaustive means the
            // next one is a compile error in this file instead.
            Op::AsIs
            | Op::Start
            | Op::Mute
            | Op::Out
            | Op::Noise
            | Op::Saw
            | Op::Reverse
            | Op::FInvert
            | Op::Ext(..)
            | Op::Follow(..)
            | Op::Tag(..)
            | Op::Keeper(..)
            | Op::CSV1d { .. }
            | Op::CSV2d { .. }
            | Op::FromSound { .. }
            | Op::FromSoundYin { .. }
            | Op::Perform { .. }
            | Op::FMOsc { .. }
            | Op::Lowpass { .. }
            | Op::Highpass { .. }
            | Op::Bandpass { .. }
            | Op::Sine { .. }
            | Op::Triangle { .. }
            | Op::Square { .. }
            | Op::Kick { .. }
            | Op::Snare { .. }
            | Op::HiHat { .. }
            | Op::Clap { .. }
            | Op::Rimshot { .. }
            | Op::Tom { .. }
            | Op::Ride { .. }
            | Op::Crash { .. }
            | Op::Shaker { .. }
            | Op::Cowbell { .. }
            | Op::AD { .. }
            | Op::Portamento { .. }
            | Op::Silence { .. }
            | Op::TransposeM { .. }
            | Op::TransposeA { .. }
            | Op::PanM { .. }
            | Op::PanA { .. }
            | Op::Gain { .. }
            | Op::Length { .. }
            | Op::Reverb { .. }
            | Op::Wavefolder { .. }
            | Op::SoftClip { .. }
            | Op::Overdrive { .. }
            | Op::Bitcrusher { .. }
            | Op::Tanh { .. } => Ok(Term::Op(self.clone())),
        }
    }
}

pub fn substitute_operations(
    operations: Vec<Term>,
    normal_form: &mut NormalForm,
    defs: &mut Defs,
) -> Result<Vec<Term>, Error> {
    let mut result = vec![];
    for term in operations {
        match term {
            Term::Nf(nf) => result.push(Term::Nf(nf)),
            Term::Op(op) => {
                let subbed = op.substitute(normal_form, defs)?;
                result.push(subbed)
            }
            Term::FunDef(_fun) => {
                return Err(Error::with_msg("Cannot get length_ratio of FunDef."))
            }
            Term::Lop(lop) => {
                let subbed = lop.substitute(normal_form, defs)?;
                result.push(subbed)
            }
            Term::Gen(r#gen) => {
                let subbed = r#gen.substitute(normal_form, defs)?;
                result.push(subbed)
            }
        }
    }

    Ok(result)
}
