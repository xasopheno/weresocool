use crate::operations::{helpers::handle_id_error, NormalForm, Normalize, Substitute};
use crate::{FunDef, Op, Term};
use scop::Defs;
use weresocool_error::Error;

pub fn insert_function_args(f: &Term, args: &[Term], defs: &mut Defs<Term>) -> Result<(), Error> {
    match f {
        Term::FunDef(fun) => {
            let FunDef { vars, .. } = fun;
            let new_scope = defs.create_uuid_scope();
            for (var, arg) in vars.iter().zip(args.iter()) {
                defs.insert(&new_scope, var.to_string(), arg.clone());
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

impl Substitute<Term> for Op {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs<Term>,
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
            Op::Compose { operations } => Ok(Term::Op(Op::Compose {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
            })),
            Op::ModulateBy { operations } => Ok(Term::Op(Op::ModulateBy {
                operations: substitute_operations(operations.to_vec(), normal_form, defs)?,
            })),
            Op::Lambda {
                term,
                input_name,
                scope,
            } => {
                if let Some(name) = input_name {
                    defs.insert(scope, name, Term::Nf(normal_form.to_owned()));
                }
                Ok(Term::Op(Op::Lambda {
                    input_name: input_name.to_owned(),
                    term: Box::new(term.substitute(normal_form, defs)?),
                    scope: scope.into(),
                }))
            }
            _ => Ok(Term::Op(self.clone())),
        }
    }
}

pub fn substitute_operations(
    operations: Vec<Term>,
    normal_form: &mut NormalForm,
    defs: &mut Defs<Term>,
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
            Term::Gen(gen) => {
                let subbed = gen.substitute(normal_form, defs)?;
                result.push(subbed)
            }
        }
    }

    Ok(result)
}
