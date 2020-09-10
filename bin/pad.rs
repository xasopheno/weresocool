use num_rational::Rational64;
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::io::{prelude::*, BufReader};
use std::sync::{Arc, Mutex};
use weresocool::data::*;
use weresocool::generation::{
    parsed_to_render::{render, write_audio_to_file},
    RenderReturn, RenderType,
};
use weresocool::interpretable::{InputType::Filename, Interpretable};
use weresocool::manager::RenderManager;
use weresocool::portaudio::real_time_render_manager;
use weresocool::write::write_composition_to_wav;
use weresocool_ast::{Defs, NormalForm, PointOp};
use weresocool_error::Error;
use weresocool_instrument::renderable::{nf_to_vec_renderable, renderables_to_render_voices};
use weresocool_instrument::{renderable::Renderable, Basis, StereoWaveform};
use weresocool_parser::Init;

fn main() -> Result<(), Error> {
    let file = std::fs::File::open("nn/output/out.csv")?;
    let reader = BufReader::new(file);

    let data: Vec<f64> = reader
        .lines()
        .map(|line| line.unwrap().parse().unwrap())
        .collect();

    let data: Vec<String> = data.iter().map(|v| format!("{:.16}", v)).collect();

    let pre_ops: Vec<Vec<String>> = data.chunks(7).map(|chunck| chunck.to_vec()).collect();
    let point_ops: Vec<PointOp> = pre_ops
        .iter()
        .map(|chunk| DataOp::from_vec_f64_string(chunk.to_vec()).to_point_op())
        .collect();
    let result: Vec<Vec<PointOp>> = point_ops.chunks(64).map(|chunck| chunck.to_vec()).collect();
    let mut nf = NormalForm::init_empty();
    nf.operations = result;

    let init: Init = Init {
        f: Rational64::new(220, 1),
        l: Rational64::new(1, 1),
        g: Rational64::new(1, 1),
        p: Rational64::new(0, 1),
    };

    let basis = Basis::from(init);
    let defs: Defs = Default::default();
    let sw = render(&basis, &nf, &defs).unwrap();
    let wav = write_composition_to_wav(sw).unwrap();
    write_audio_to_file(&wav, "test", "wav");

    // let renderables = nf_to_vec_renderable(&nf, &defs, &Basis::from(init))?;
    // let mut voices = renderables_to_render_voices(renderables);

    // let mut result = StereoWaveform::new(0);
    // loop {
    // let batch: Vec<StereoWaveform> = voices
    // .par_iter_mut()
    // .filter_map(|voice| voice.render_batch(default_settings().buffer_size, None))
    // .collect();

    // if !batch.is_empty() {
    // let stereo_waveform = sum_all_waveforms(batch);
    // result.append(stereo_waveform);
    // } else {
    // break;
    // }
    // }

    // Ok(result)

    // fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
    // let mut result: StereoWaveform = StereoWaveform::new(0);

    // for op in self.iter() {
    // if op.samples > 0 {
    // let stereo_waveform = op.clone().render(oscillator, offset);
    // result.append(stereo_waveform);
    // }
    // }

    // result
    // }

    // let sws = render_voices.map(|rv| rv.to)

    // let render_manager = Arc::new(Mutex::new(RenderManager::init(render_voices)));

    // let mut stream = real_time_render_manager(Arc::clone(&render_manager))?;
    // stream.start()?;

    // while let true = stream.is_active()? {}
    // stream.stop()?;
    Ok(())
}

fn _generate_data() -> Result<(), Error> {
    let (min_state, max_state) = find_min_max_from_dir()?;
    let normalizer = Normalizer::from_min_max(min_state, max_state);

    let render_return = Filename("application/extraResources/demo/monica.socool")
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

        let filename = format!("nn/data/monica/monica_{:0>10}.socool.csv", i);
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
