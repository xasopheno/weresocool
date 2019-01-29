pub mod normalize {
    extern crate num_rational;
    extern crate rand;
    use crate::ast::{Op, OpOrNfTable, OscType};
    use crate::operations::helpers::*;
    use crate::operations::{GetLengthRatio, NormalForm, Normalize};
    use num_rational::Ratio;
    use rand::prelude::*;

    impl Normalize for Op {
        fn apply_to_normal_form(&self, input: &mut NormalForm, table: &OpOrNfTable) {
            match self {
                Op::Id(id_vec) => {
                    for seq in input.operations.iter_mut() {
                        for p_op in seq {
                            let id = id_vec.join(".");
                            p_op.names.insert(id);
                        }
                    }
                    handle_id_error(id_vec.to_vec(), table).apply_to_normal_form(input, table);
                }

                Op::AsIs => {}

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

                //                Op::Choice { operations } => {
                //                    let choice = rand::thread_rng().choose(&operations).unwrap();
                //                    choice.apply_to_normal_form(input, table)
                //                }
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
