use crate::{
    generation::{to_csv, to_json},
    instrument::{Basis, Normalize, Oscillator, StereoWaveform},
    renderable::{nf_to_vec_renderable, RenderOp, Renderable},
    settings::default_settings,
    ui::{banner, printed},
    write::write_composition_to_wav,
};
use num_rational::Rational64;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use weresocool_ast::{NormalForm, Normalize as NormalizeOp, Term, TermTable};
use weresocool_error::Error;
use weresocool_parser::ParsedComposition;

#[derive(Clone, PartialEq, Debug)]
pub enum RenderType {
    Json4d,
    Csv1d,
    NfBasisAndTable,
    StereoWaveform,
    Wav,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RenderReturn {
    Json4d(String),
    Csv1d(String),
    StereoWaveform(StereoWaveform),
    NfBasisAndTable(NormalForm, Basis, TermTable),
    Wav(String),
}

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

pub fn parsed_to_render(
    filename: &str,
    parsed_composition: ParsedComposition,
    return_type: RenderType,
) -> Result<RenderReturn, Error> {
    let parsed_main = parsed_composition.table.get("main").unwrap();

    let nf = match parsed_main {
        Term::Nf(nf) => nf,
        Term::Op(_) => panic!("main is not in Normal Form for some terrible reason."),
        Term::FunDef(_) => unimplemented!(),
        Term::Lop(_) => unimplemented!(),
        Term::Lnf(_) => unimplemented!(),
    };

    let basis = Basis::from(parsed_composition.init);

    match return_type {
        RenderType::NfBasisAndTable => Ok(RenderReturn::NfBasisAndTable(
            nf.clone(),
            basis,
            parsed_composition.table,
        )),
        RenderType::Json4d => {
            to_json(
                &basis,
                nf,
                &parsed_composition.table.clone(),
                filename.to_string(),
            )?;
            Ok(RenderReturn::Json4d("json".to_string()))
        }
        RenderType::Csv1d => {
            to_csv(
                &basis,
                nf,
                &parsed_composition.table.clone(),
                filename.to_string(),
            )?;
            Ok(RenderReturn::Csv1d("json".to_string()))
        }
        RenderType::StereoWaveform | RenderType::Wav => {
            let stereo_waveform = render(&basis, nf, &parsed_composition.table);
            if return_type == RenderType::StereoWaveform {
                Ok(RenderReturn::StereoWaveform(stereo_waveform))
            } else {
                let result = to_wav(stereo_waveform, filename.to_string());
                Ok(RenderReturn::Wav(result))
            }
        }
    }
}

pub fn render(basis: &Basis, composition: &NormalForm, table: &TermTable) -> StereoWaveform {
    let mut normal_form = NormalForm::init();

    println!("\nGenerating Composition ");
    composition.apply_to_normal_form(&mut normal_form, table);
    let render_ops = nf_to_vec_renderable(composition, table, basis);

    let vec_wav = generate_waveforms(render_ops, true);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    result
}

pub fn to_wav(composition: StereoWaveform, filename: String) -> String {
    banner("Printing".to_string(), filename.clone());
    write_composition_to_wav(composition, filename.as_str(), true, true);
    printed("WAV".to_string());
    "composition.wav".to_string()
}

fn create_pb_instance(n: usize) -> Arc<Mutex<ProgressBar<std::io::Stdout>>> {
    let mut pb = ProgressBar::new(n as u64);
    pb.format("╢w♬░╟");
    pb.message("Rendering:  ");
    Arc::new(Mutex::new(pb))
}

pub fn generate_waveforms(
    mut vec_sequences: Vec<Vec<RenderOp>>,
    show: bool,
) -> Vec<StereoWaveform> {
    if show {
        println!("Rendering {:?} waveforms", vec_sequences.len());
    }
    let pb = create_pb_instance(vec_sequences.len());

    let vec_wav = vec_sequences
        .par_iter_mut()
        .map(|ref mut vec_render_op: &mut Vec<RenderOp>| {
            pb.lock().unwrap().add(1 as u64);
            let mut osc = Oscillator::init(&default_settings());
            vec_render_op.render(&mut osc, None)
        })
        .collect();

    pb.lock().unwrap().finish_print(&"".to_string());

    vec_wav
}

pub fn sum_all_waveforms(mut vec_wav: Vec<StereoWaveform>) -> StereoWaveform {
    let mut result = StereoWaveform::new(0);

    sort_vecs(&mut vec_wav);

    let max_len = vec_wav[0].l_buffer.len();

    result.l_buffer.resize(max_len, 0.0);
    result.r_buffer.resize(max_len, 0.0);

    for wav in vec_wav {
        sum_vec(&mut result.l_buffer, &wav.l_buffer[..]);
        sum_vec(&mut result.r_buffer, &wav.r_buffer[..])
    }

    result
}

fn sort_vecs(vec_wav: &mut Vec<StereoWaveform>) {
    vec_wav.sort_unstable_by(|a, b| b.l_buffer.len().cmp(&a.l_buffer.len()));
}

pub fn sum_vec(a: &mut Vec<f64>, b: &[f64]) {
    for (ai, bi) in a.iter_mut().zip(b) {
        *ai += *bi;
    }
}
