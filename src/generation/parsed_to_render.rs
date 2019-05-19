use crate::instrument::{Basis, Normalize, Oscillator, StereoWaveform};
use crate::render::{Render, RenderPointOp};
use crate::settings::default_settings;
use crate::ui::{banner, printed};
use crate::write::write_composition_to_wav;
use num_rational::Rational64;
use pbr::ProgressBar;
use rayon::prelude::*;
use socool_ast::{NormalForm, Normalize as NormalizeOp, OpOrNfTable, PointOp};
use std::sync::{Arc, Mutex};

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

pub fn render(origin: &Basis, composition: &NormalForm, table: &OpOrNfTable) -> StereoWaveform {
    let mut normal_form = NormalForm::init();

    println!("\nGenerating Composition ");
    composition.apply_to_normal_form(&mut normal_form, table);

    let vec_wav = generate_waveforms(&origin, normal_form.operations, true);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    result
}

pub fn render_mic(point_op: &PointOp, origin: Basis, osc: &mut Oscillator) -> StereoWaveform {
    let result = point_op.clone().render(&origin, osc, None);
    result
}

pub fn to_wav(composition: StereoWaveform, filename: String) {
    banner("Printing".to_string(), filename.clone());
    write_composition_to_wav(composition, &filename);
    printed("WAV".to_string());
}

fn create_pb_instance(n: usize) -> Arc<Mutex<ProgressBar<std::io::Stdout>>> {
    let mut pb = ProgressBar::new(n as u64);
    pb.format("╢w♬░╟");
    pb.message("Rendering:  ");
    Arc::new(Mutex::new(pb))
}

pub fn generate_waveforms(
    origin: &Basis,
    mut vec_sequences: Vec<Vec<PointOp>>,
    show: bool,
) -> Vec<StereoWaveform> {
    if show {
        println!("Rendering {:?} waveforms", vec_sequences.len());
    }
    let pb = create_pb_instance(vec_sequences.len());

    let vec_wav = vec_sequences
        .par_iter_mut()
        .map(|ref mut vec_point_op: &mut Vec<PointOp>| {
            pb.lock().unwrap().add(1 as u64);
            let mut osc = Oscillator::init(&default_settings());
            vec_point_op.render(&origin, &mut osc)
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
