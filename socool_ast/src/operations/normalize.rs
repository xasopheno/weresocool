pub mod normalize {
    extern crate num_rational;
    extern crate rand;
    use crate::ast::{Op, OpOrNf, OpOrNf::*, OpOrNfTable, OscType};
    use crate::operations::helpers::*;
    use crate::operations::{ArgMap, GetLengthRatio, NormalForm, Normalize, Substitute};
    use num_rational::Ratio;
    use rand::prelude::*;
    use std::collections::HashMap;

    fn get_fn_arg_map(f: OpOrNf, args: &Vec<OpOrNf>) -> ArgMap {
        let mut arg_map: ArgMap = HashMap::new();
        match f {
            OpOrNf::Op(fun) => match fun {
                Op::FunctionDef {
                    op_or_nf: _,
                    name: _,
                    vars,
                } => {
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

                    let mut nf = NormalForm::init();
                    let op_to_apply = op_to_apply.substitute(normal_form, table, arg_map);
                    op_to_apply.apply_to_normal_form(&mut nf, table);
                    let named_applied = nf.clone() * named;
                    //
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
                    operations: substitute_operations(
                        operations.to_vec(),
                        normal_form,
                        table,
                        arg_map,
                    ),
                }),
                Op::Overlay { operations } => OpOrNf::Op(Op::Overlay {
                    operations: substitute_operations(
                        operations.to_vec(),
                        normal_form,
                        table,
                        arg_map,
                    ),
                }),
                Op::Compose { operations } => OpOrNf::Op(Op::Compose {
                    operations: substitute_operations(
                        operations.to_vec(),
                        normal_form,
                        table,
                        arg_map,
                    ),
                }),
                Op::Choice { operations } => OpOrNf::Op(Op::Choice {
                    operations: substitute_operations(
                        operations.to_vec(),
                        normal_form,
                        table,
                        arg_map,
                    ),
                }),
                Op::ModulateBy { operations } => OpOrNf::Op(Op::Choice {
                    operations: substitute_operations(
                        operations.to_vec(),
                        normal_form,
                        table,
                        arg_map,
                    ),
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
                OpOrNf::Op(op) => match op.clone() {
                    Op::Fid(name) => {
                        let sub = arg_map.get(&name).unwrap();
                        let subbed = op.substitute(normal_form, table, arg_map);
                        result.push(subbed)
                    }
                    _ => {
                        let subbed = op.substitute(normal_form, table, arg_map);
                        result.push(subbed)
                    }
                },
            }
        }

        result
    }

    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: &mut NormalForm, table: &OpOrNfTable) {
            match self {
                Op::AsIs => {}

                Op::Id(id) => {
                    handle_id_error(id.to_string(), table).apply_to_normal_form(input, table);
                }
                //
                Op::Fid(_) => {}
                Op::FunctionDef {
                    name: _,
                    vars: _,
                    op_or_nf: _,
                } => {}
                Op::FunctionCall { name, args } => {
                    let f = handle_id_error(name.to_string(), table);
                    let arg_map = get_fn_arg_map(f.clone(), args);

                    match f {
                        OpOrNf::Op(fun) => match fun {
                            Op::FunctionDef {
                                op_or_nf,
                                name: _,
                                vars: _,
                            } => match *op_or_nf {
                                OpOrNf::Op(op) => {
                                    let result_op = op.substitute(input, table, &arg_map);
                                    result_op.apply_to_normal_form(input, table)
                                }
                                OpOrNf::Nf(_) => {
                                    panic!("Function stored in NormalForm");
                                }
                            },
                            _ => panic!("Function Stored not FunctionDef"),
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
                            point_op.fm = Ratio::new(0, 1);
                            point_op.fa = Ratio::new(0, 1);
                            point_op.g = Ratio::new(0, 1);
                            point_op.l = point_op.l * m;
                        }
                    }

                    input.length_ratio = *m;
                }

                Op::Choice { operations } => {
                    let choice = rand::thread_rng().choose(&operations).unwrap();
                    choice.apply_to_normal_form(input, table)
                }

                Op::Sequence { operations } => {
                    let mut result = NormalForm::init_empty();

                    for op in operations {
                        let mut input_clone = input.clone();
                        op.apply_to_normal_form(&mut input_clone, table);
                        result = join_sequence(result, input_clone);
                    }

                    *input = result
                }

                Op::Compose { operations } => {
                    for op in operations {
                        op.apply_to_normal_form(input, table);
                    }
                }

                Op::WithLengthRatioOf {
                    with_length_of,
                    main,
                } => {
                    let target_length = with_length_of.get_length_ratio(table);
                    let main_length = main.get_length_ratio(table);
                    let ratio = target_length / main_length;
                    let new_op = Op::Length { m: ratio };

                    new_op.apply_to_normal_form(input, table);

                    input.length_ratio = target_length;
                }

                Op::Focus {
                    name,
                    main,
                    op_to_apply,
                } => {
                    main.apply_to_normal_form(input, table);
                    let (named, rest) = input.clone().partition(name.to_string());
                    let mut nf = NormalForm::init();
                    op_to_apply.apply_to_normal_form(&mut nf, table);
                    let named_applied = nf * named;

                    let mut result = NormalForm::init();
                    Op::Overlay {
                        operations: vec![Nf(named_applied), Nf(rest)],
                    }
                    .apply_to_normal_form(&mut result, table);

                    *input = result
                }

                Op::ModulateBy { operations } => {
                    let mut modulator = NormalForm::init_empty();

                    for op in operations {
                        let mut nf = NormalForm::init();
                        op.apply_to_normal_form(&mut nf, table);
                        modulator = join_sequence(modulator, nf);
                    }

                    Op::Length {
                        m: input.length_ratio / modulator.length_ratio,
                    }
                    .apply_to_normal_form(&mut modulator, table);

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
                    let normal_forms: Vec<NormalForm> = operations
                        .iter()
                        .map(|op| {
                            let mut input_clone = input.clone();
                            op.apply_to_normal_form(&mut input_clone, table);
                            input_clone
                        })
                        .collect();

                    let max_lr = normal_forms
                        .iter()
                        .map(|nf| nf.length_ratio)
                        .max()
                        .expect("Normalize, max_lr not working");

                    let mut result = vec![];

                    for mut nf in normal_forms {
                        pad_length(&mut nf, max_lr, table);
                        result.append(&mut nf.operations);
                    }

                    *input = NormalForm {
                        operations: result,
                        length_ratio: max_lr,
                    };
                }
            }
        }
    }
}
