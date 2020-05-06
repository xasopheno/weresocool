use crate::operations::{helpers::handle_id_error, ArgMap, NormalForm, Normalize, Substitute};
use crate::{Defs, FunDef, Op, Term};
use std::collections::HashMap;
use weresocool_error::Error;

pub fn get_fn_arg_map(f: Term, args: &[Term]) -> Result<ArgMap, Error> {
    let mut arg_map: ArgMap = HashMap::new();
    match f {
        Term::FunDef(fun) => {
            let FunDef { vars, .. } = fun;
            for (var, arg) in vars.iter().zip(args.iter()) {
                arg_map.insert(var.to_string(), arg.clone());
            }
        }
        _ => {
            println!("FunctionCall does not point to FunctionDef");
            return Err(Error::with_msg(
                "FunctionCall does not point to FunctionDef",
            ));
        }
    }

    Ok(arg_map)
}

impl Substitute for Op {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &Defs,
        arg_map: &ArgMap,
    ) -> Result<Term, Error> {
        match self {
            Op::Id(id) => handle_id_error(id.to_string(), defs, Some(arg_map)),

            Op::WithLengthRatioOf {
                main,
                with_length_of,
            } => {
                let main = main.substitute(normal_form, defs, arg_map)?;
                let with_length_of = with_length_of.substitute(normal_form, defs, arg_map)?;

                Ok(Term::Op(Op::WithLengthRatioOf {
                    main: Box::new(main),
                    with_length_of: Box::new(with_length_of),
                }))
            }

            Op::Focus {
                name,
                main,
                op_to_apply,
            } => {
                let mut nf = NormalForm::init();
                let m = main.substitute(normal_form, defs, arg_map)?;
                m.apply_to_normal_form(&mut nf, defs)?;
                let (named, rest) = nf.partition(name.to_string());

                let op_to_apply = op_to_apply.substitute(normal_form, defs, arg_map)?;

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
                args: substitute_operations(args.to_vec(), normal_form, defs, arg_map)?,
            })),
            Op::Sequence { operations } => Ok(Term::Op(Op::Sequence {
                operations: substitute_operations(operations.to_vec(), normal_form, defs, arg_map)?,
            })),
            Op::Overlay { operations } => Ok(Term::Op(Op::Overlay {
                operations: substitute_operations(operations.to_vec(), normal_form, defs, arg_map)?,
            })),
            Op::Compose { operations } => Ok(Term::Op(Op::Compose {
                operations: substitute_operations(operations.to_vec(), normal_form, defs, arg_map)?,
            })),
            Op::Choice { operations } => Ok(Term::Op(Op::Choice {
                operations: substitute_operations(operations.to_vec(), normal_form, defs, arg_map)?,
            })),
            Op::ModulateBy { operations } => Ok(Term::Op(Op::Choice {
                operations: substitute_operations(operations.to_vec(), normal_form, defs, arg_map)?,
            })),
            _ => Ok(Term::Op(self.clone())),
        }
    }
}

pub fn substitute_operations(
    operations: Vec<Term>,
    normal_form: &mut NormalForm,
    defs: &Defs,
    arg_map: &ArgMap,
) -> Result<Vec<Term>, Error> {
    let mut result = vec![];
    for term in operations {
        match term {
            Term::Nf(nf) => result.push(Term::Nf(nf)),
            Term::Op(op) => {
                let subbed = op.substitute(normal_form, defs, arg_map)?;
                result.push(subbed)
            }
            Term::FunDef(_fun) => {
                return Err(Error::with_msg("Cannot get length_ratio of FunDef."))
            }
            Term::Lop(lop) => {
                let subbed = lop.substitute(normal_form, defs, arg_map)?;
                result.push(subbed)
            }
        }
    }

    Ok(result)
}
