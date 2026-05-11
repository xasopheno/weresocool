use crate::{
    generation::{csv::get_length_op4d_1d, to_csv, to_json_file, Normalizer}, manager::render_op_to_normalized_op4d_list,
    ui::printed,
    write::{write_composition_to_mp3, write_composition_to_wav},
};
use std::path::PathBuf;

use super::Op4D;
#[cfg(feature = "app")]
use pbr::ProgressBar;
#[cfg(feature = "app")]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fs::File;
use std::io::Write;
use std::path::Path;
#[cfg(feature = "app")]
use std::sync::{Arc, Mutex};
use weresocool_ast::{NormalForm, Term, Defs};
use weresocool_error::{Error, IdError};
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, renderables_to_render_voices, RenderOp, Renderable,
};
use weresocool_instrument::{Basis, Oscillator, StereoWaveform};
use weresocool_parser::ParsedComposition;
use weresocool_shared::Settings;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum WavType {
    Wav {
        cli: bool,
        output_dir: PathBuf,
    },
    Mp3 {
        cli: bool,
        output_dir: PathBuf,
    },
    #[cfg(feature = "app")]
    OggVorbis {
        cli: bool,
        output_dir: PathBuf,
    },
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RenderType {
    Json4d { cli: bool, output_dir: PathBuf },
    Csv1d { cli: bool, output_dir: PathBuf },
    NfBasisAndTable,
    StereoWaveform,
    Wav(WavType),
    Stems { cli: bool, output_dir: PathBuf },
    AudioVisual,
    Visual,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
/// A stem is an audio file of a NormalForm which has been solo'd
/// and the audio rendered only contains operations with the name
/// in the NameSet.
pub struct Stem {
    /// Stem name
    pub name: String,
    /// Stem audio
    pub audio: Vec<u8>,
}

#[derive(Clone, PartialEq, Debug)]
/// AudioVisual is the datatype for audiovisualization
pub struct AudioVisual {
    /// Composition name
    pub name: String,
    /// length of seconds of composition
    pub length: f32,
    /// audio data
    pub audio: Vec<u8>,
    /// visual data
    pub visual: Vec<Op4D>,

    pub defs: Defs,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Visual {
    /// Composition name
    pub name: String,
    /// length of seconds of composition
    pub length: f32,
    /// visual data
    pub visual: Vec<Op4D>,
}

#[derive(Clone, PartialEq, Debug)]
#[allow(clippy::large_enum_variant)]
/// Target of a render
pub enum RenderReturn {
    Json4d(String),
    Csv1d(String),
    StereoWaveform(StereoWaveform),
    /// NormalForm, Basis, and Definition Table
    NfBasisAndTable(NormalForm, Basis, Defs),
    /// Wav or Mp3
    Wav(Vec<u8>),
    /// A vector of audio solo'd by names
    Stems(Vec<Stem>),
    /// Data for audiovisualization
    AudioVisual(AudioVisual),
}

impl TryFrom<RenderReturn> for Vec<u8> {
    type Error = Error;
    fn try_from(r: RenderReturn) -> Result<Self, Self::Error> {
        match r {
            RenderReturn::Wav(audio) => Ok(audio),
            _ => Err(Error::with_msg(
                "Can only produce Vec<u8> from RenderReturn::Wav",
            )),
        }
    }
}

/// Generate a render of a parsed composition in the target render_type.
/// The filename is only used to for naming things when needed.
pub fn parsed_to_render(
    filename: &str,
    mut parsed_composition: ParsedComposition,
    return_type: RenderType,
) -> Result<RenderReturn, Error> {
    let parsed_main = parsed_composition.defs.ops.get("main");

    if parsed_main.is_none() {
        return Err(IdError { id: "main".into() }.into_error());
    };

    let nf = match parsed_main.unwrap().to_owned() {
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
        _ => unimplemented!(),
    };

    let basis = Basis::from_init(&parsed_composition.init);

    match return_type {
        RenderType::Visual => {
            let (visual, length) = make_visuals(&basis, &nf, &mut parsed_composition)?;
            Ok(RenderReturn::AudioVisual(AudioVisual {
                name: filename.to_string(),
                length: length as f32,
                visual,
                audio: vec![],
                defs: parsed_composition.defs,
            }))
        }
        RenderType::AudioVisual => {
            // Build renderables once and share between audio and visual generation.
            // Previously `render()` and `make_visuals()` each called `nf_to_vec_renderable`
            // independently — doubling that ~350ms cost on drum_sounds.socool.
            let renderables = nf_to_vec_renderable(&nf, &mut parsed_composition.defs, &basis)?;

            // Visualization reads renderables by reference, so it must run BEFORE we
            // move them into the audio voices.
            let (visual, length) = make_visuals_from_renderables(&renderables);

            let voices = renderables_to_render_voices(renderables);
            let stereo_waveform = render_from_voices(voices);
            let audio = write_composition_to_wav(stereo_waveform)?;
            Ok(RenderReturn::AudioVisual(AudioVisual {
                name: filename.to_string(),
                length: length as f32,
                audio,
                visual,
                defs: parsed_composition.defs,
            }))
        }

        RenderType::Stems { cli, output_dir } => {
            let nf_names = nf.names();
            let names = parsed_composition.defs.ops.stems.clone();
            if !names.is_subset(&nf_names) {
                let difference = names
                    .difference(&nf_names)
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ");

                return Err(Error::with_msg(format!(
                    "Stem names not found in composition: {}",
                    difference
                )));
            }

            if names.is_empty() {
                return Err(Error::with_msg("No stems to render"));
            }

            let mut result: Vec<Stem> = vec![];
            println!("Rendering:");
            for name in names {
                println!("\t{}", &name);
                let mut n = nf.clone();
                n.solo_ops_by_name(&name);
                let stereo_waveform = render(&basis, &n, &mut parsed_composition.defs)?;

                result.push(Stem {
                    name,
                    audio: write_composition_to_wav(stereo_waveform)?,
                });
            }

            #[cfg(feature = "app")]
            if cli {
                stems_to_zip(&result, filename, output_dir).unwrap();
            }
            Ok(RenderReturn::Stems(result))
        }
        RenderType::NfBasisAndTable => Ok(RenderReturn::NfBasisAndTable(
            nf,
            basis,
            parsed_composition.defs,
        )),
        RenderType::Json4d { output_dir, .. } => {
            to_json_file(
                &basis,
                &nf,
                &mut parsed_composition.defs,
                filename.to_string(),
                output_dir,
            )?;
            Ok(RenderReturn::Json4d("json".to_string()))
        }
        RenderType::Csv1d { output_dir, .. } => {
            to_csv(
                &basis,
                &nf,
                &mut parsed_composition.defs,
                filename.to_string(),
                output_dir,
            )?;
            Ok(RenderReturn::Csv1d("csv".to_string()))
        }
        RenderType::StereoWaveform => {
            let stereo_waveform = render(&basis, &nf, &mut parsed_composition.defs)?;
            Ok(RenderReturn::StereoWaveform(stereo_waveform))
        }
        RenderType::Wav(wav_type) => match wav_type {
            WavType::Mp3 {
                cli,
                mut output_dir,
            } => {
                let stereo_waveform = render(&basis, &nf, &mut parsed_composition.defs)?;
                let render_return = RenderReturn::Wav(write_composition_to_mp3(stereo_waveform)?);
                if cli {
                    let audio: Vec<u8> = Vec::try_from(render_return.clone())?;
                    let f = filename_to_renderpath(filename)?;
                    // println!("filename: {}", &output_dir);
                    output_dir.push(format!("{}.mp3", f));
                    write_audio_to_file(&audio, output_dir)?;
                };
                Ok(render_return)
            }
            WavType::Wav {
                cli,
                mut output_dir,
            } => {
                let stereo_waveform = render(&basis, &nf, &mut parsed_composition.defs)?;
                let render_return = RenderReturn::Wav(write_composition_to_wav(stereo_waveform)?);
                if cli {
                    let audio: Vec<u8> = Vec::try_from(render_return.clone())?;
                    let f = filename_to_renderpath(filename)?;
                    output_dir.push(format!("{}.wav", f));
                    write_audio_to_file(&audio, output_dir)?;
                };
                Ok(render_return)
            }
            #[cfg(feature = "app")]
            WavType::OggVorbis { cli: _, output_dir: _ } 
            => {
                todo!()
            }

            // #[cfg(feature = "app")]
            // WavType::OggVorbis {
                // cli,
                // mut output_dir,
            // } => {
                // let stereo_waveform = render(&basis, &nf, &mut parsed_composition.defs)?;
                // let render_return =
                    // RenderReturn::Wav(weresocool_vorbis::encode_lr_channels_to_ogg_vorbis(
                        // stereo_waveform.l_buffer,
                        // stereo_waveform.r_buffer,
                    // ));
                // if cli {
                    // let audio: Vec<u8> = Vec::try_from(render_return.clone())?;
                    // let f = filename_to_renderpath(filename);
                    // output_dir.push(format!("{}.ogg", f));
                    // write_audio_to_file(&audio, output_dir);
                // };
                // Ok(render_return)
            // }
        },
    }
}

fn filename_to_renderpath(filename: &str) -> Result<String, Error> {
    let path = Path::new(filename)
        .file_stem()
        .ok_or_else(|| Error::with_msg(format!("Invalid filename: no file stem in '{}'", filename)))?;
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::with_msg(format!("Invalid UTF-8 in filename: '{:?}'", path)))?;
    Ok(path_str.to_string())
}

pub fn write_audio_to_file(audio: &[u8], filename: PathBuf) -> Result<(), Error> {
    let mut file = File::create(&filename)
        .map_err(|e| Error::with_msg(format!("Failed to create file '{}': {}", filename.display(), e)))?;
    file.write_all(audio)
        .map_err(|e| Error::with_msg(format!("Failed to write to file '{}': {}", filename.display(), e)))?;
    printed(filename.display().to_string());
    Ok(())
}

pub fn render(
    basis: &Basis,
    composition: &NormalForm,
    defs: &mut Defs,
) -> Result<StereoWaveform, Error> {
    let renderables = nf_to_vec_renderable(composition, defs, basis)?;
    let voices = renderables_to_render_voices(renderables);
    Ok(render_from_voices(voices))
}

/// Drive a set of pre-built render voices through the buffer loop.
/// Factored out of `render` so callers that already have voices in hand
/// (e.g. the AudioVisual path, which shares renderables with visualization)
/// can avoid rebuilding them.
pub fn render_from_voices(mut voices: Vec<weresocool_instrument::renderable::RenderVoice>) -> StereoWaveform {
    let buffer_size = Settings::global().buffer_size;

    let mut result = StereoWaveform::new(0);
    loop {
        #[cfg(feature = "app")]
        let iter = voices.par_iter_mut();
        #[cfg(feature = "wasm")]
        let iter = voices.iter_mut();
        let batch: Vec<StereoWaveform> = iter
            .filter_map(|voice| voice.render_batch(buffer_size, None))
            .collect();

        if !batch.is_empty() {
            let stereo_waveform = sum_all_waveforms(batch);
            result.append(stereo_waveform);
        } else {
            break;
        }
    }

    result
}

#[cfg(feature = "app")]
fn stems_to_zip(
    stems: &[Stem],
    filename: &str,
    mut output_dir: PathBuf,
) -> zip::result::ZipResult<()> {
    output_dir.push(format!(
        "{}.stems.zip",
        Path::new(filename)
            .file_name()
            .expect("No filename")
            .to_string_lossy()
    ));
    let file = File::create(std::path::Path::new(&output_dir))?;
    let mut zip = zip::ZipWriter::new(file);

    let options: zip::write::FileOptions<()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for stem in stems {
        zip.start_file(format!("{}.stem.wav", stem.name), options)?;
        zip.write_all(&stem.audio)?;
    }

    // Apply the changes you've made.
    // Dropping the `ZipWriter` will have the same effect, but may silently fail
    zip.finish()?;
    Ok(())
}

#[cfg(feature = "app")]
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
    #[cfg(feature = "app")]
    let pb = create_pb_instance(vec_sequences.len());

    #[cfg(feature = "app")]
    let iter = vec_sequences.par_iter_mut();
    #[cfg(feature = "wasm")]
    let iter = vec_sequences.iter_mut();

    let vec_wav = iter
        .map(|ref mut vec_render_op: &mut Vec<RenderOp>| {
            #[cfg(feature = "app")]
            if let Ok(mut pb) = pb.lock() {
                pb.add(1_u64);
            }
            let mut osc = Oscillator::init();
            vec_render_op.render(&mut osc, None)
        })
        .collect();

    #[cfg(feature = "app")]
    if let Ok(mut pb) = pb.lock() {
        pb.finish_print("");
    }

    vec_wav
}

fn make_visuals(
    basis: &Basis,
    nf: &NormalForm,
    parsed_composition: &mut ParsedComposition,
) -> Result<(Vec<Op4D>, f64), Error> {
    let renderables = nf_to_vec_renderable(nf, &mut parsed_composition.defs, basis)?;
    Ok(make_visuals_from_renderables(&renderables))
}

/// Same as `make_visuals` but consumes already-built renderables. The AudioVisual path
/// uses this so it only calls `nf_to_vec_renderable` once instead of twice.
pub fn make_visuals_from_renderables(renderables: &[Vec<RenderOp>]) -> (Vec<Op4D>, f64) {
    let normalizer = Normalizer::default();

    let mut visual: Vec<Op4D> = renderables
        .iter()
        .flat_map(|voice| voice.iter())
        .flat_map(|op| render_op_to_normalized_op4d_list(op, &normalizer, 1.0 / 30.0))
        .collect();

    let length = get_length_op4d_1d(&visual);

    // Handle NaN values gracefully - treat them as equal for sorting
    visual.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));

    (visual, length)
}

/// Sum a vec of StereoWaveform to a single stereo_waveform.
pub fn sum_all_waveforms(mut vec_wav: Vec<StereoWaveform>) -> StereoWaveform {
    // Sort the vectors by length
    sort_vecs(&mut vec_wav);

    // Get the length of the longest vector
    let max_len = vec_wav[0].l_buffer.len();

    let mut result = StereoWaveform::new(max_len);

    for wav in vec_wav {
        sum_vec(&mut result.l_buffer, &wav.l_buffer[..]);
        sum_vec(&mut result.r_buffer, &wav.r_buffer[..])
    }

    result
}

/// Sort a vec of StereoWaveform by length. Assumes both channels have the same
/// buffer length
fn sort_vecs(vec_wav: &mut [StereoWaveform]) {
    vec_wav.sort_unstable_by(|a, b| b.l_buffer.len().cmp(&a.l_buffer.len()));
}

/// Sum two vectors. Assumes vector a is longer than or of the same length
/// as vector b.
pub fn sum_vec(a: &mut [f64], b: &[f64]) {
    for (ai, bi) in a.iter_mut().zip(b) {
        *ai += *bi;
    }
}
