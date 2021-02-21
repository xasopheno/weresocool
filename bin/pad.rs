use num_rational::Rational64;
use weresocool::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    ui::were_so_cool_logo,
};
use weresocool_ast::{NormalForm, PointOp};

use weresocool_error::Error;

fn print_lens(nf: &NormalForm) {
    for (i, voice) in nf.operations.iter().enumerate() {
        dbg!("________", i);
        for op in voice {
            dbg!(op.l);
        }
    }
}

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let filename = "songs/template_1.socool";

    let render_return = Filename(filename).make(RenderType::NfBasisAndTable, None)?;
    let (nf, _basis, _table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let result = divide_into_n_equal_lengths(nf, 3);

    dbg!(result);
    Ok(())
}

/// Divides a NormalForm into n equal parts lengthwise.
/// This is accomplished one voice at a time.
pub fn divide_into_n_equal_lengths(mut nf: NormalForm, n_divisions: usize) -> Vec<NormalForm> {
    // This function should take a usize, but an i64 is more practical for computation.
    let n_divisions = n_divisions as i64;
    // length of an nth.
    let lr_division = nf.length_ratio / n_divisions;
    let mut result: Vec<Vec<Vec<PointOp>>> = vec![];
    // Prepare n bins ahead of time.
    for _ in 0..n_divisions {
        result.push(vec![])
    }
    for voice in nf.operations.iter_mut() {
        let mut division_counter = 0;
        let mut voice_idx = 0;
        let mut target_len = lr_division;
        while division_counter < n_divisions {
            let mut voice_division_result: Vec<PointOp> = vec![];
            let mut lr_accumulator = lr_division * division_counter;
            loop {
                // If we've reached the target_len of the division
                // move on to next division or, finally, voice.
                if lr_accumulator >= target_len || voice_idx >= voice.len() {
                    break;
                };
                // If we haven't reached our target, add this op to
                // in it's entirety.
                if lr_accumulator + voice[voice_idx].l <= target_len {
                    voice_division_result.push(voice[voice_idx].clone());
                    lr_accumulator += voice[voice_idx].l;
                    voice_idx += 1;
                } else {
                    // Otherwise, add an an op with a partial length
                    let mut op_clone = voice[voice_idx].clone();
                    let op_l = target_len - lr_accumulator;
                    op_clone.l = op_l;
                    voice_division_result.push(op_clone);
                    if voice[voice_idx].l - op_l == Rational64::from_integer(0) {
                        // If the op is now empty, move on.
                        voice_idx += 1
                    } else {
                        // If not, replace the l in the current op with
                        // the remainder.
                        voice[voice_idx].l = voice[voice_idx].l - op_l;
                    }
                    lr_accumulator += op_l;
                }
            }
            result[division_counter as usize].push(voice_division_result);
            division_counter += 1;
            target_len = lr_division * (division_counter + 1);
        }
    }
    // Build NormalForms from the Vec<NormalForm.operations> above.
    // All of the length_ratios should be the same.
    result
        .iter()
        .map(|operations| NormalForm::init_with_operations_and_lr(operations.to_vec(), lr_division))
        .collect()
}
