use crate::datagen::{csv2d_to_normalform, mod_1d::csv1d_to_normalform};
use crate::follow::normalize::ToNF;
use crate::operations::Rational64;
use crate::operations::{
    helpers::*, substitute::insert_function_args, GetLengthRatio, NormalForm, Normalize, Substitute, Defs,
};
use crate::{Distortion, FunDef, Op, OscType, Term, Term::*, wgsl::rational_to_f32};
use num_rational::Ratio;
use num_traits::CheckedMul;
use weresocool_error::Error;
use weresocool_filter::BiquadFilterDef;
use weresocool_shared::lossy_rational_mul;

impl Normalize for Op {
    #[allow(clippy::cognitive_complexity)]
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<(), Error> {
        match self {
            // EXTENSION OPS — one arm; all bodies live in ext::normalize_ext_op
            // (the extension module owns its own normalization).
            Op::Ext(ext_op) => {
                crate::operations::ext::normalize_ext_op(ext_op, input, defs);
            }
            Op::Follow(follow) => {
                let fnf = follow.to_nf(Some(Default::default()));
                input.fmap_mut(|op| {
                    op.follows.push(fnf.clone());
                });
            }
            Op::AsIs => {}
            // Positional playback-start marker. Capture the cumulative
            // beat-time the NF has reached at this point — that's the
            // time the renderer should seek to before the first frame.
            // Multiple `Start` markers in the same NF overwrite this
            // field, so the LAST one in the chain wins (which IS the
            // composer's intent: "move the marker around to scrub").
            // No-op for length: see get_length_ratio.rs (returns 1/1).
            // Caveat: only useful at the top level of `main`. Inside
            // Sequence/Lerp/Choose the sub-NF's start_at is dropped
            // when composed into the outer NF (see the start_at: None
            // fallback in the Mul/MulAssign/Overlay branches). Document.
            Op::Start => {
                input.start_at = Some(input.length_ratio);
            }
            // None (source keyword) / Mute (variant name): kill — silence.
            // Same effect as `Gm 0`; preserves length. The semantic
            // difference from Out is that None doesn't zero length, so
            // `None | Lm 3` is "3 base units of silence."
            Op::Mute => {
                input.fmap_mut(|op| {
                    op.g = Ratio::new(0, 1);
                });
            }
            Op::Out => {
                input.fmap_mut(|op| {
                    op.is_out = true;
                    op.fm = Ratio::new(0, 1);
                    op.fa = Ratio::new(0, 1);
                    op.g = Ratio::new(0, 1);
                    op.l = Ratio::new(0, 1)
                });
            }
            Op::Lambda {
                term,
                input_name,
                scope,
            } => {
                if let Some(name) = input_name {
                    defs.ops.insert(scope, name, Term::Nf(input.to_owned()));
                }
                let mut nf = NormalForm::init();
                term.apply_to_normal_form(&mut nf, defs)?;
                *input = nf;
            }

            Op::Id(id) => {
                handle_id_error(id, defs)?.apply_to_normal_form(input, defs)?;
            }

            Op::FMOsc { defs } => input.fmap_mut(|op| {
                op.osc_type = OscType::Fm {
                    defs: defs.to_owned(),
                }
            }),
            Op::Lowpass {
                hash,
                cutoff_frequency,
                q_factor,
            } => {
                let filter_def = BiquadFilterDef {
                    hash: hash.to_owned(),
                    filter_type: weresocool_filter::BiquadFilterType::Lowpass,
                    cutoff_frequency: *cutoff_frequency,
                    q_factor: *q_factor,
                };
                input.fmap_mut(|op| op.filters.push(filter_def.clone()));
            }

            Op::Highpass {
                hash,
                cutoff_frequency,
                q_factor,
            } => {
                let filter_def = BiquadFilterDef {
                    hash: hash.to_owned(),
                    filter_type: weresocool_filter::BiquadFilterType::Highpass,
                    cutoff_frequency: *cutoff_frequency,
                    q_factor: *q_factor,
                };
                input.fmap_mut(|op| op.filters.push(filter_def.clone()));
            }

            Op::Bandpass {
                hash,
                cutoff_frequency,
                q_factor,
            } => {
                let filter_def = BiquadFilterDef {
                    hash: hash.to_owned(),
                    filter_type: weresocool_filter::BiquadFilterType::Bandpass,
                    cutoff_frequency: *cutoff_frequency,
                    q_factor: *q_factor,
                };
                input.fmap_mut(|op| op.filters.push(filter_def.clone()));
            }

            Op::CSV1d { path, scale } => {
                csv1d_to_normalform(path, *scale)?.apply_to_normal_form(input, defs)?;
            }

            Op::CSV2d { path, scales } => {
                csv2d_to_normalform(path, scales.clone())?.apply_to_normal_form(input, defs)?;
            }

            #[cfg(feature = "weresocool-from-sound")]
            Op::FromSound { path, voices, fps } => {
                crate::datagen::from_sound::from_sound_to_normalform(path, *voices, *fps)?
                    .apply_to_normal_form(input, defs)?;
            }
            #[cfg(not(feature = "weresocool-from-sound"))]
            Op::FromSound { .. } => {
                return Err(weresocool_error::Error::with_msg(
                    "FromSound requires the weresocool-from-sound feature",
                ));
            }

            #[cfg(feature = "weresocool_analyze")]
            Op::FromSoundYin { path, fps } => {
                crate::datagen::from_sound::from_sound_yin_to_normalform(path, *fps)?
                    .apply_to_normal_form(input, defs)?;
            }
            #[cfg(not(feature = "weresocool_analyze"))]
            Op::FromSoundYin { .. } => {
                return Err(weresocool_error::Error::with_msg(
                    "FromSoundYin requires the weresocool_analyze feature",
                ));
            }

            // Resolve a named recording from the registry the host seeded.
            // Hit: apply its NormalForm like an inline def. Miss: do NOT
            // error — leave the NF untouched (silent placeholder) and record
            // the name so the host can pre-arm a track for it.
            Op::Perform { name } => {
                let recording = defs.recordings.get(name).cloned();
                match recording {
                    Some(nf) => nf.apply_to_normal_form(input, defs)?,
                    None => {
                        defs.pending_performs.insert(name.clone());
                    }
                }
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

            Op::Keeper(_) => {
                // Keeper acts like AsIs - it's just a marker for Slice
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

            Op::Wavefolder {
                threshold,
                stages,
                input_gain,
                output_gain,
            } => {
                input.fmap_mut(|op| {
                    op.distortions.push(Distortion::Wavefolder {
                        threshold: *threshold,
                        stages: (*stages).clamp(1, 8) as u8,
                        input_gain: *input_gain,
                        output_gain: *output_gain,
                    });
                });
            }

            Op::SoftClip { threshold, input_gain, output_gain } => {
                input.fmap_mut(|op| {
                    op.distortions.push(Distortion::SoftClip {
                        threshold: *threshold,
                        input_gain: *input_gain,
                        output_gain: *output_gain,
                    });
                });
            }

            Op::Overdrive { input_gain, output_gain } => {
                input.fmap_mut(|op| {
                    op.distortions.push(Distortion::Overdrive {
                        input_gain: *input_gain,
                        output_gain: *output_gain,
                    });
                });
            }

            Op::Bitcrusher { bits, input_gain, output_gain } => {
                input.fmap_mut(|op| {
                    op.distortions.push(Distortion::Bitcrusher {
                        bits: (*bits).clamp(1, 16) as u8,
                        input_gain: *input_gain,
                        output_gain: *output_gain,
                    });
                });
            }

            Op::Tanh { input_gain, output_gain } => {
                input.fmap_mut(|op| {
                    op.distortions.push(Distortion::Tanh {
                        input_gain: *input_gain,
                        output_gain: *output_gain,
                    });
                });
            }

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

            Op::Saw => input.fmap_mut(|op| op.osc_type = OscType::Saw),

            Op::Square { width } => {
                input.fmap_mut(|op| op.osc_type = OscType::Square { width: *width })
            }

            Op::Noise => input.fmap_mut(|op| op.osc_type = OscType::Noise),

            Op::Kick { params } => input.fmap_mut(|op| op.osc_type = OscType::Kick { params: params.clone() }),

            Op::Snare { params } => input.fmap_mut(|op| op.osc_type = OscType::Snare { params: params.clone() }),

            Op::HiHat { open, params } => input.fmap_mut(|op| op.osc_type = OscType::HiHat { open: *open, params: params.clone() }),

            Op::Clap { params } => input.fmap_mut(|op| op.osc_type = OscType::Clap { params: params.clone() }),

            Op::Rimshot { params } => input.fmap_mut(|op| op.osc_type = OscType::Rimshot { params: params.clone() }),

            Op::Tom { params } => input.fmap_mut(|op| op.osc_type = OscType::Tom { params: params.clone() }),

            Op::Ride { params } => input.fmap_mut(|op| op.osc_type = OscType::Ride { params: params.clone() }),

            Op::Crash { params } => input.fmap_mut(|op| op.osc_type = OscType::Crash { params: params.clone() }),

            Op::Shaker { params } => input.fmap_mut(|op| op.osc_type = OscType::Shaker { params: params.clone() }),

            Op::Cowbell { params } => input.fmap_mut(|op| op.osc_type = OscType::Cowbell { params: params.clone() }),

            Op::TransposeM { m, .. } => input.fmap_mut(|op| {
                op.fm = op
                    .fm
                    .checked_mul(m)
                    .unwrap_or_else(|| lossy_rational_mul(op.fm, *m))
            }),

            Op::TransposeA { a, .. } => input.fmap_mut(|op| {
                op.fa += a;
            }),

            Op::PanA { a, .. } => input.fmap_mut(|op| {
                op.pa += a;
            }),

            Op::PanM { m, .. } => input.fmap_mut(|op| {
                op.pm = op
                    .pm
                    .checked_mul(m)
                    .unwrap_or_else(|| lossy_rational_mul(op.pm, *m))
            }),

            Op::Gain { m, .. } => input.fmap_mut(|op| {
                op.g =
                    op.g.checked_mul(m)
                        .unwrap_or_else(|| lossy_rational_mul(op.g, *m))
            }),

            Op::Length { m, .. } => {
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

            Op::Sequence { operations, .. } => {
                let mut result = NormalForm::init_empty();
                result.operations.reserve(operations.len() * input.operations.len());
                let saved_rand_ctx = defs.rand_ctx;
                let last_i = operations.len().saturating_sub(1);
                for (i, op) in operations.iter().enumerate() {
                    // Each sequence item gets a unique rand_ctx based on its index
                    defs.rand_ctx = saved_rand_ctx.child_ord(i as u64);
                    // For the final iteration, move `input` instead of cloning —
                    // we overwrite it with `result` immediately after the loop.
                    let mut working = if i == last_i {
                        std::mem::replace(input, NormalForm::init_empty())
                    } else {
                        input.clone()
                    };
                    op.apply_to_normal_form(&mut working, defs)?;

                    // Propagate `Start` markers up through Sequence:
                    // when one of the operands carried a positional
                    // `| Start`, its inner `start_at` is the beat-time
                    // of the marker WITHIN that operand. Translate to
                    // outer time by adding `result.length_ratio` —
                    // i.e. the cumulative offset of everything Seq has
                    // already concatenated. Last-wins, so a marker
                    // later in the list overwrites earlier ones.
                    //
                    // Without this, dropping `| Start` inside a Seq
                    // would silently do nothing (`join_sequence` resets
                    // `start_at` to None on the fresh result NF). With
                    // it, the natural cull.socool pattern works:
                    //   Seq [
                    //       Fm 0 | Lm 3,
                    //       Fm 3/2 | Lm 2 | Start,    -- play from here
                    //       Fm 4/3 | Lm 2,
                    //   ]
                    let working_start_at = working.start_at;
                    let offset_before_working = result.length_ratio;
                    result = join_sequence(result, working);
                    if let Some(inner) = working_start_at {
                        result.start_at = Some(offset_before_working + inner);
                    }
                }
                defs.rand_ctx = saved_rand_ctx;

                *input = result
            }

            Op::Compose { operations, .. } => {
                let saved_rand_ctx = defs.rand_ctx;
                for (i, op) in operations.iter().enumerate() {
                    // Each composed operation gets a unique rand_ctx based on its position
                    defs.rand_ctx = saved_rand_ctx.child_ord(i as u64);
                    op.apply_to_normal_form(input, defs)?;
                }
                defs.rand_ctx = saved_rand_ctx;
            }












            Op::WithLengthRatioOf {
                with_length_of,
                main: _,
            } => {
                // Use the actual length of the already-normalized input
                // (not re-evaluated from main, which would make different random choices)
                let main_length = input.length_ratio;
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

            Op::ModulateBy { operations, output } => {
                // Build the modulator and track which operations are "keepers" ($a syntax)
                let mut modulator = NormalForm::init_empty();
                let mut keeper_indices: Vec<usize> = vec![];
                let mut keeper_names: Vec<String> = vec![];

                for (idx, op) in operations.iter().enumerate() {
                    let mut nf = NormalForm::init();

                    // Extract keeper name if this is a keeper
                    let keeper_name = match op {
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
                        keeper_indices.push(idx);
                        keeper_names.push(name);
                    }

                    op.apply_to_normal_form(&mut nf, defs)?;
                    modulator = join_sequence(modulator, nf);
                }

                // Scale modulator to match input length
                Op::Length {
                    m: input.length_ratio / modulator.length_ratio,
                }
                .apply_to_normal_form(&mut modulator, defs)?;

                // If we have keepers, use slice behavior; otherwise normal modulate
                if keeper_indices.is_empty() {
                    // Normal ModulateBy behavior
                    let result_operations: Vec<_> = modulator
                        .operations
                        .iter()
                        .flat_map(|modulation_line| {
                            input
                                .operations
                                .iter()
                                .map(|input_line| modulate(input_line, modulation_line))
                                .collect::<Vec<_>>()
                        })
                        .collect();

                    let mut result = NormalForm::init_empty();
                    result.operations = result_operations;
                    result.length_ratio = input.length_ratio;

                    *input = result
                } else {
                    // Slice behavior - only return keeper pieces
                    // Compute the lengths of each original operation (after scaling)
                    let mut op_lengths: Vec<Rational64> = vec![];
                    let scale = input.length_ratio / operations.iter().try_fold(
                        Ratio::from_integer(0),
                        |acc, op| -> Result<Rational64, Error> {
                            Ok(acc + op.get_length_ratio(input, defs)?)
                        }
                    )?;

                    for op in operations.iter() {
                        let len = op.get_length_ratio(input, defs)? * scale;
                        op_lengths.push(len);
                    }

                    if let Some(output_ops) = output {
                        // Output mapping mode - extract each keeper separately and bind by name
                        let scope = defs.ops.create_uuid_scope();

                        // Extract each keeper slice individually
                        for (i, &keeper_idx) in keeper_indices.iter().enumerate() {
                            let single_keeper = vec![keeper_idx];
                            let keeper_ops: Vec<_> = modulator
                                .operations
                                .iter()
                                .flat_map(|modulation_line| {
                                    input
                                        .operations
                                        .iter()
                                        .map(|input_line| {
                                            slice_modulate(input_line, modulation_line, &op_lengths, &single_keeper)
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .collect();

                            let mut keeper_nf = NormalForm::init_empty();
                            keeper_nf.operations = keeper_ops;
                            keeper_nf.length_ratio = op_lengths[keeper_idx];

                            // Bind this keeper to its name in the scope
                            defs.ops.insert(&scope, &keeper_names[i], Nf(keeper_nf));
                        }

                        // Now resolve the output operations against the bound names
                        let mut result = NormalForm::init_empty();
                        for output_op in output_ops {
                            let mut nf = NormalForm::init();
                            output_op.apply_to_normal_form(&mut nf, defs)?;
                            result = join_sequence(result, nf);
                        }

                        *input = result
                    } else {
                        // No output mapping - return all keepers in order
                        let result_operations: Vec<_> = modulator
                            .operations
                            .iter()
                            .flat_map(|modulation_line| {
                                input
                                    .operations
                                    .iter()
                                    .map(|input_line| {
                                        slice_modulate(input_line, modulation_line, &op_lengths, &keeper_indices)
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect();

                        // Calculate the length of just the keeper pieces
                        let keeper_length: Rational64 = keeper_indices
                            .iter()
                            .map(|&idx| op_lengths[idx])
                            .sum();

                        let mut result = NormalForm::init_empty();
                        result.operations = result_operations;
                        result.length_ratio = keeper_length;

                        *input = result
                    }
                }
            }

            Op::Choose { operations } => {
                if operations.is_empty() {
                    return Err(Error::with_msg("Empty Choose!"));
                }
                // Use rand_ctx to select an index
                let n = std::num::NonZeroUsize::new(operations.len())
                    .ok_or_else(|| Error::with_msg("Choose with zero operations"))?;
                let idx = defs.rand_ctx.index(n, 0);
                // Normalize the selected operation
                operations[idx].apply_to_normal_form(input, defs)?;
            }

            Op::Repeat { operations, count } => {
                let count = *count as usize;
                let mut result = NormalForm::init_empty();
                result.operations.reserve(count * input.operations.len());
                let saved_rand_ctx = defs.rand_ctx;

                let last_i = count.saturating_sub(1);
                for i in 0..count {
                    // Bump epoch for each iteration so Choose gets fresh randomness
                    defs.rand_ctx = defs.rand_ctx.bump_epoch();
                    // For the final iteration, move `input` instead of cloning —
                    // we overwrite it with `result` immediately after the loop.
                    let mut working = if i == last_i {
                        std::mem::replace(input, NormalForm::init_empty())
                    } else {
                        input.clone()
                    };

                    for op in operations {
                        op.apply_to_normal_form(&mut working, defs)?;
                    }

                    result = join_sequence(result, working);
                }

                defs.rand_ctx = saved_rand_ctx;
                *input = result
            }

            Op::Zip { operations } => {
                if operations.is_empty() {
                    return Err(Error::with_msg("Empty Zip!"));
                }
                // Each operand gets its own rand_ctx branch, same as Seq and
                // Overlay, so a `Choose` inside one operand does not shift
                // when a sibling operand changes.
                let saved_rand_ctx = defs.rand_ctx;
                let result = zip_terms(operations, input, defs)?;
                defs.rand_ctx = saved_rand_ctx;
                *input = result;
            }

            Op::Overlay { operations, .. } => {
                if operations.is_empty() {
                    return Err(Error::with_msg("Empty Overlay!"));
                }

                let saved_rand_ctx = defs.rand_ctx;
                let last_i = operations.len() - 1;
                let mut normal_forms: Vec<NormalForm> = Vec::with_capacity(operations.len());
                for (i, op) in operations.iter().enumerate() {
                    // Each overlay item gets a unique rand_ctx based on its index
                    defs.rand_ctx = saved_rand_ctx.child_ord(i as u64);
                    // For the final iteration, move `input` instead of cloning —
                    // `*input` is unconditionally overwritten at the end of this arm.
                    let mut working = if i == last_i {
                        std::mem::replace(input, NormalForm::init_empty())
                    } else {
                        input.clone()
                    };
                    op.apply_to_normal_form(&mut working, defs)?;
                    normal_forms.push(working);
                }
                defs.rand_ctx = saved_rand_ctx;

                let max_lr = normal_forms
                    .iter()
                    .map(|nf: &NormalForm| nf.length_ratio)
                    .max()
                    .ok_or_else(|| Error::with_msg("Failed to compute max length ratio"))?;

                let total_ops: usize = normal_forms.iter().map(|nf| nf.operations.len()).sum();
                let mut result = Vec::with_capacity(total_ops);

                for mut nf in normal_forms {
                    pad_length(&mut nf, max_lr, defs)?;
                    result.append(&mut nf.operations);
                }

                *input = NormalForm {
                    operations: result,
                    length_ratio: max_lr,
                    // This is the Overlay branch — overlay doesn't
                    // emit a Start of its own; inherit nothing here.
                    // The caller's Start (if any) survives because it
                    // applies AFTER overlay returns.
                    start_at: None,
                };
            }
        }
        Ok(())
    }
}
