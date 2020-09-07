use rayon::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use weresocool::data::*;
use weresocool::generation::{RenderReturn, RenderType};
use weresocool::interpretable::{InputType::Filename, Interpretable};
use weresocool_error::Error;

fn main() -> Result<(), Error> {
    let (min_state, max_state) = find_min_max_from_dir()?;
    let normalizer = Normalizer::from_min_max(min_state, max_state);

    let render_return = Filename("application/extraResources/demo/how_to_rest.socool")
        .make(RenderType::NfBasisAndTable, None)?;
    // let render_return =
    // Filename("songs/template.socool").make(RenderType::NfBasisAndTable, None)?;
    let (nf, _, _) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };

    let normalized: Vec<Vec<DataOp>> = nf_to_normalized_vec_data_op(&nf, &normalizer);
    let voice_len = 64;
    let op_len = 7;
    // let n_voices = &normalized.len();

    let mut i = 0;
    let min_len = voice_len - 10;
    // let min_len = 0;
    let mut next = normalized;

    loop {
        if !is_not_empty(&next, min_len) {
            break;
        };

        let filename = format!("nn/data/how_to_rest/how_to_rest_{:0>10}.socool.csv", i);
        dbg!(&filename);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            // .open("nn/real_data.csv")
            .open(filename)
            .unwrap();

        file.write(
            format!(
                "{}, {}\n",
                // n_voices.to_string(),
                voice_len.to_string(),
                op_len.to_string(),
            )
            .as_bytes(),
        )?;

        i += 1;
        if i % 1000 == 0 {
            dbg!(i);
        };
        // if i > 20 {
        // break;
        // };

        let result = process_normalized(&next, voice_len);
        let nnops = result
            .par_iter()
            .map(|voice| {
                voice
                    .par_iter()
                    .map(|op| op.to_nnop())
                    .collect::<Vec<NNOp>>()
            })
            .collect::<Vec<Vec<NNOp>>>();

        nnops.iter().for_each(|voice| {
            voice
                .iter()
                .for_each(|nnop| nnop.write_to_file(&mut file).unwrap())
        });

        let len = shortest_first_element(&next);
        next = make_next(len, &next);
    }

    Ok(())
}
