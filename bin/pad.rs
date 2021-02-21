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
    let (mut nf, _basis, _table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let n = 4;
    let lr = nf.length_ratio / Rational64::from_integer(n);
    // dbg!(lr);
    // dbg!(nf);
    let mut result: Vec<NormalForm> = vec![];
    let mut outer_counter = 1;
    let mut target_len = lr;
    while outer_counter < n + 1 {
        dbg!(target_len);
        dbg!(outer_counter);
        let mut slice_operations = vec![];
        for voice in nf.operations.iter_mut() {
            let mut voice_result: Vec<PointOp> = vec![];
            let mut current = lr * outer_counter - 1;
            let mut i = 0;
            loop {
                dbg!(i);
                if i >= voice.len() || current >= target_len {
                    break;
                };
                if current + voice[i].l <= target_len {
                    voice_result.push(voice[i].clone());
                    current += voice[i].l;
                    i += 1;
                } else {
                    let mut op_clone = voice[i].clone();
                    let op_l = target_len - current;
                    op_clone.l = op_l;
                    voice_result.push(op_clone);
                    dbg!(voice[i].l - op_l);
                    if voice[i].l - op_l == Rational64::from_integer(0) {
                        i += 1
                    } else {
                        voice[i].l = voice[i].l - op_l;
                    }
                    current += op_l;
                }
            }
            slice_operations.push(voice_result);
        }
        result.push(NormalForm::init_with_operations_and_lr(
            slice_operations,
            lr,
        ));
        outer_counter += 1;
        target_len = lr * outer_counter;
        dbg!(&result.len());
    }

    Ok(())
}
