use crate::{
    generation::{to_csv, to_json},
    instrument::{Basis, Oscillator, StereoWaveform},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices, RenderOp, Renderable},
    settings::{default_settings, Settings},
    ui::{banner, printed},
    write::{write_composition_to_mp3, write_composition_to_wav},
};
use num_rational::Rational64;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use weresocool_ast::{Defs, NormalForm, Term};
use weresocool_error::{Error, IdError};
use weresocool_parser::ParsedComposition;

const SETTINGS: Settings = default_settings();

#[derive(Clone, PartialEq, Debug)]
pub enum WavType {
    Wav { cli: bool },
    MP3 { cli: bool },
}

#[derive(Clone, PartialEq, Debug)]
pub enum RenderType {
    Json4d,
    Csv1d,
    NfBasisAndTable,
    StereoWaveform,
    Wav(WavType),
}

#[derive(Clone, PartialEq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum RenderReturn {
    Json4d(String),
    Csv1d(String),
    StereoWaveform(StereoWaveform),
    NfBasisAndTable(NormalForm, Basis, Defs),
    Wav(Vec<u8>),
}

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

pub fn parsed_to_render(
    filename: &str,
    parsed_composition: ParsedComposition,
    return_type: RenderType,
) -> Result<RenderReturn, Error> {
    let parsed_main = parsed_composition.defs.terms.get("main");

    let nf = match parsed_main {
        Some(main) => match main {
            Term::Nf(nf) => nf,
            Term::Op(_) => {
                println!("main is not in Normal Form for some terrible reason.");
                return Err(Error::with_msg("Unrecoverable Error"));
            }
            Term::FunDef(_) => {
                println!("main as function not yet supported.");
                return Err(Error::with_msg("main as function not yet supported"));
            }
            Term::Lop(_) => {
                println!("main as list not yet supported.");
                return Err(Error::with_msg("main as list not yet supported"));
            }
        },
        None => {
            return Err(IdError {
                id: "main".to_string(),
            }
            .into_error())
        }
    };

    let basis = Basis::from(parsed_composition.init);

    match return_type {
        RenderType::NfBasisAndTable => Ok(RenderReturn::NfBasisAndTable(
            nf.clone(),
            basis,
            parsed_composition.defs,
        )),
        RenderType::Json4d => {
            to_json(
                &basis,
                nf,
                &parsed_composition.defs.clone(),
                filename.to_string(),
            )?;
            Ok(RenderReturn::Json4d("json".to_string()))
        }
        RenderType::Csv1d => {
            to_csv(
                &basis,
                nf,
                &parsed_composition.defs.clone(),
                filename.to_string(),
            )?;
            Ok(RenderReturn::Csv1d("json".to_string()))
        }
        RenderType::StereoWaveform => {
            let stereo_waveform = render(&basis, nf, &parsed_composition.defs)?;
            Ok(RenderReturn::StereoWaveform(stereo_waveform))
        }
        RenderType::Wav(wav_type) => match wav_type {
            WavType::MP3 { cli } => {
                let stereo_waveform = render(&basis, nf, &parsed_composition.defs)?;
                Ok(RenderReturn::Wav(write_composition_to_mp3(
                    stereo_waveform,
                    filename,
                )?))
            }
            WavType::Wav { cli } => {
                let stereo_waveform = render(&basis, nf, &parsed_composition.defs)?;
                Ok(RenderReturn::Wav(write_composition_to_wav(
                    stereo_waveform,
                    filename,
                )?))
            }
        },
    }
}

pub fn render(
    basis: &Basis,
    composition: &NormalForm,
    defs: &Defs,
) -> Result<StereoWaveform, Error> {
    let renderables = nf_to_vec_renderable(&composition, &defs, &basis)?;
    let mut voices = renderables_to_render_voices(renderables);

    let mut result = StereoWaveform::new(0);
    loop {
        let batch: Vec<StereoWaveform> = voices
            .par_iter_mut()
            .filter_map(|voice| voice.render_batch(SETTINGS.buffer_size, None))
            .collect();

        if !batch.is_empty() {
            let stereo_waveform = sum_all_waveforms(batch);
            result.append(stereo_waveform);
        } else {
            break;
        }
    }

    Ok(result)
}

pub fn to_wav(composition: StereoWaveform, filename: String) -> Vec<u8> {
    banner("Printing".to_string(), filename.clone());
    let composition = write_composition_to_mp3(composition, filename.as_str());
    printed("WAV".to_string());
    unimplemented!();
    // composition
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
