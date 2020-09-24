use num_rational::Rational64;
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{prelude::*, BufReader};
use weresocool::data::*;
use weresocool::generation::{
    parsed_to_render::{render, write_audio_to_file},
    RenderReturn, RenderType,
};
use weresocool::interpretable::{InputType::Filename, Interpretable};
use weresocool::write::write_composition_to_wav;
use weresocool_ast::{Defs, NormalForm, PointOp};
use weresocool_error::Error;
use weresocool_instrument::Basis;
use weresocool_parser::Init;

fn main() -> Result<(), Error> {
    // Test file before processing
    // let file = std::fs::File::open("nn/data/slice/slice_0000000100.socool.csv")?;

    // Test file after processing
    let file = std::fs::File::open("nn/output/0000_000000002.csv")?;
    let reader = BufReader::new(file);

    let (min_state, max_state) = find_min_max_from_dir()?;
    let normalizer = Normalizer::from_min_max(min_state, max_state);
    let op_len = 1;

    // Test file before processing
    // let mut data: Vec<f64> = vec![];
    // let lines: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();
    // for (i, line) in lines.iter().enumerate() {
    // if i > 0 {
    // for val in line.split(",") {
    // data.push(val.trim().parse().unwrap())
    // }
    // }
    // }

    // Test File after processing
    let data: Vec<f64> = reader
        .lines()
        .map(|line| line.unwrap().parse().unwrap())
        .collect();

    let data: Vec<String> = data.iter().map(|v| format!("{:.16}", v)).collect();
    let pre_ops: Vec<Vec<String>> = data.chunks(op_len).map(|chunck| chunck.to_vec()).collect();
    let point_ops: Vec<PointOp> = pre_ops
        .iter()
        .map(|chunk| {
            // dbg!(&chunk);
            let mut op = DataOp::from_vec_f64_string(chunk.to_vec());
            op.denormalize(&normalizer);
            op.to_point_op()
        })
        .collect();
    let result: Vec<Vec<PointOp>> = point_ops.chunks(64).map(|chunck| chunck.to_vec()).collect();
    dbg!(&result.len());
    dbg!(&result[0].len());
    // for i in 0..result.len() {
    // if result[i].len() != 64 {
    // dbg![i];
    // dbg!("?????", result[i].len());
    // }
    // }
    // dbg!(&result);
    let mut nf = NormalForm::init_empty();
    // nf.operations[0] = result[0].clone();
    // nf.operations = vec![result[0].clone(), result[1].clone()];
    nf.operations = vec![result[0].clone()];

    let init: Init = Init {
        f: Rational64::new(230, 1),
        l: Rational64::new(1, 1),
        g: Rational64::new(1, 1),
        p: Rational64::new(0, 1),
    };

    let basis = Basis::from(init);
    let defs: Defs = Default::default();
    let sw = render(&basis, &nf, &defs)?;
    let wav = write_composition_to_wav(sw)?;
    write_audio_to_file(&wav, "test", "wav");

    Ok(())
}

fn _main() -> Result<(), Error> {
    let (min_state, max_state) = find_min_max_from_dir()?;
    let normalizer = Normalizer::from_min_max(min_state, max_state);
    let f = "simple";

    // let render_return = Filename(format!("application/extraResources/demo/{}.socool", f).as_str())
    // .make(RenderType::NfBasisAndTable, None)?;
    let render_return =
        Filename("nn/new_train/simple.socool").make(RenderType::NfBasisAndTable, None)?;
    let (nf, _, _) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };

    let normalized: Vec<Vec<DataOp>> = nf_to_normalized_vec_data_op(&nf, &normalizer);
    let voice_len = 64;
    let op_len = 7;
    // let _n_voices = &normalized.len();

    let mut i = 0;
    let min_len = voice_len;
    // let min_len = 2;

    let mut next = normalized;

    loop {
        if !is_not_empty(&next, min_len) {
            break;
        };

        let filename = format!("nn/data/{}/{}_{:0>10}.socool.csv", f, f, i);
        if i % 100 == 0 {
            dbg!(&filename);
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)
            .unwrap();

        file.write(format!("{}, {}\n", voice_len.to_string(), op_len.to_string(),).as_bytes())?;

        i += 1;
        if i % 1000 == 0 {
            dbg!(i);
        };
        // if i > 20 {
        // break;
        // };
        // dbg!(&next);

        let result = process_normalized(&next.clone(), voice_len);
        let nnops = result
            .iter()
            .map(|voice| voice.iter().map(|op| op.to_nnop()).collect::<Vec<NNOp>>())
            .collect::<Vec<Vec<NNOp>>>();

        nnops.iter().for_each(|voice| {
            voice
                .iter()
                .for_each(|nnop| nnop.write_to_file(&mut file).unwrap())
        });

        let len = shortest_first_element(&next);
        next = make_next(len, &next);
        next = pad_voices(&next, 64);
    }

    Ok(())
}
