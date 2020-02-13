use crate::ast::{FunDef, Op, Term, TermTable};
use crate::operations::{helpers::handle_id_error, ArgMap, NormalForm, Normalize, Substitute};
use std::collections::HashMap;

pub fn get_fn_arg_map(f: Term, args: &[Term]) -> ArgMap {
    let mut arg_map: ArgMap = HashMap::new();
    match f {
        Term::FunDef(fun) => match fun {
            FunDef { vars, .. } => {
                for (var, arg) in vars.iter().zip(args.iter()) {
                    arg_map.insert(var.to_string(), arg.clone());
                }
            }
            _ => panic!("Function Stored not FunctionDef"),
        },
        _ => {
            panic!("Function stored in NormalForm");
        }
    }

    arg_map
}

impl Substitute for Op {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        table: &TermTable,
        arg_map: &ArgMap,
    ) -> Term {
        match self {
            Op::Id(id) => {
                let value = arg_map.get(&id.clone());
                let result = match value {
                    Some(sub) => sub.clone(),
                    None => handle_id_error(id.to_string(), table),
                };

                result
            }

            Op::WithLengthRatioOf {
                main,
                with_length_of,
            } => {
                let main = main.substitute(normal_form, table, arg_map);
                let with_length_of = with_length_of.substitute(normal_form, table, arg_map);

                Term::Op(Op::WithLengthRatioOf {
                    main: Box::new(main),
                    with_length_of: Box::new(with_length_of),
                })
            }

            Op::Focus {
                name,
                main,
                op_to_apply,
            } => {
                let mut nf = NormalForm::init();
                let m = main.substitute(normal_form, table, arg_map);
                m.apply_to_normal_form(&mut nf, table);
                let (named, rest) = nf.partition(name.to_string());

                let op_to_apply = op_to_apply.substitute(normal_form, table, arg_map);

                let mut nf = NormalForm::init();
                op_to_apply.apply_to_normal_form(&mut nf, table);
                let named_applied = nf * named;

                let mut result = NormalForm::init();

                Op::Overlay {
                    operations: vec![Term::Nf(named_applied), Term::Nf(rest)],
                }
                .apply_to_normal_form(&mut result, table);

                Term::Nf(result)
            }
            Op::FunctionCall { name, args } => Term::Op(Op::FunctionCall {
                name: name.to_string(),
                args: substitute_operations(args.to_vec(), normal_form, table, arg_map),
            }),
            Op::Sequence { operations } => Term::Op(Op::Sequence {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::Overlay { operations } => Term::Op(Op::Overlay {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::Compose { operations } => Term::Op(Op::Compose {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::Choice { operations } => Term::Op(Op::Choice {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::ModulateBy { operations } => Term::Op(Op::Choice {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            _ => Term::Op(self.clone()),
        }
    }
}

fn substitute_operations(
    operations: Vec<Term>,
    normal_form: &mut NormalForm,
    table: &TermTable,
    arg_map: &ArgMap,
) -> Vec<Term> {
    let mut result = vec![];
    for term in operations {
        match term {
            Term::Nf(nf) => result.push(Term::Nf(nf)),
            Term::Op(op) => {
                let subbed = op.substitute(normal_form, table, arg_map);
                result.push(subbed)
            }
            Term::FunDef(_fun) => {
                unimplemented!();
            }
        }
    }

    result
}
