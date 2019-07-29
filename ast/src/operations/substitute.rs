use crate::ast::{Op, OpOrNf, OpOrNf::*, OpOrNfTable};
use crate::operations::{ArgMap, NormalForm, Normalize, Substitute};
use std::collections::HashMap;

pub fn get_fn_arg_map(f: OpOrNf, args: &[OpOrNf]) -> ArgMap {
    let mut arg_map: ArgMap = HashMap::new();
    match f {
        OpOrNf::Op(fun) => match fun {
            Op::FunctionDef { vars, .. } => {
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
        table: &OpOrNfTable,
        arg_map: &ArgMap,
    ) -> OpOrNf {
        match self {
            Op::Fid(name) => {
                let sub = arg_map.get(&name.clone()).unwrap();
                sub.clone()
            }
            Op::WithLengthRatioOf {
                main,
                with_length_of,
            } => {
                let main = main.substitute(normal_form, table, arg_map);
                let with_length_of = with_length_of.substitute(normal_form, table, arg_map);

                Op(Op::WithLengthRatioOf {
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
                    operations: vec![Nf(named_applied), Nf(rest)],
                }
                .apply_to_normal_form(&mut result, table);

                Nf(result)
            }
            Op::FunctionCall { name, args } => OpOrNf::Op(Op::FunctionCall {
                name: name.to_string(),
                args: substitute_operations(args.to_vec(), normal_form, table, arg_map),
            }),
            Op::Sequence { operations } => OpOrNf::Op(Op::Sequence {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::Overlay { operations } => OpOrNf::Op(Op::Overlay {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::Compose { operations } => OpOrNf::Op(Op::Compose {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::Choice { operations } => OpOrNf::Op(Op::Choice {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            Op::ModulateBy { operations } => OpOrNf::Op(Op::Choice {
                operations: substitute_operations(operations.to_vec(), normal_form, table, arg_map),
            }),
            _ => OpOrNf::Op(self.clone()),
        }
    }
}

fn substitute_operations(
    operations: Vec<OpOrNf>,
    normal_form: &mut NormalForm,
    table: &OpOrNfTable,
    arg_map: &ArgMap,
) -> Vec<OpOrNf> {
    let mut result = vec![];
    for op_or_nf in operations {
        match op_or_nf {
            OpOrNf::Nf(nf) => result.push(OpOrNf::Nf(nf)),
            OpOrNf::Op(op) => {
                    let subbed = op.substitute(normal_form, table, arg_map);
                    result.push(subbed)
            }
        }
    }

    result
}
