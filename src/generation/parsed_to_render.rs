extern crate num_rational;
extern crate pbr;
extern crate rayon;
extern crate socool_parser;
use render::Render;
use instrument::{
    oscillator::{Oscillator, OscillatorBasis},
    stereo_waveform::{Normalize, StereoWaveform},
};
use num_rational::Rational64;
use operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use pbr::ProgressBar;
use rayon::prelude::*;
use settings::get_default_app_settings;
use socool_parser::{ast::Op, parser::Init};
use std::sync::{Arc, Mutex};
use ui::{banner, printed};
use write::write_composition_to_wav;

type PointOpSequences = Vec<Vec<PointOp>>;
type VecWav = Vec<StereoWaveform>;

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

//pub fn event_from_init(init: Init) -> Event {
//    Event::init(
//        r_to_f64(init.f),
//        r_to_f64(init.g),
//        r_to_f64(init.p),
//        r_to_f64(init.l),
//    )
//}

pub fn render(composition: &Op, init: Init) -> StereoWaveform {
    let mut normal_form = NormalForm::init();

    println!("\nGenerating Composition ");
    composition.apply_to_normal_form(&mut normal_form);

    //    let e = event_from_init(init);

    //    let norm_ev = generate_events(normal_form.operations, e);
    let vec_wav = generate_waveforms(init, normal_form.operations);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    result
}

pub fn render_mic(composition: &Op) -> StereoWaveform {
//    let mut normal_form = NormalForm::init();
    //
    //    composition.apply_to_normal_form(&mut normal_form);
    ////    let norm_ev = generate_events(normal_form.operations, render);
    //    let vec_wav = generate_waveforms(normal_form.operations);
    //    let mut result = sum_all_waveforms(vec_wav);
    //    result.normalize();
    //
    //    result
    StereoWaveform::new(2048)
}

pub fn to_wav(composition: StereoWaveform, filename: String) {
    banner("Printing".to_string(), filename);
    write_composition_to_wav(composition);
    printed("WAV".to_string());
}

pub fn to_json(composition: &Op, init: Init, filename: String) {
    println!("Not working");
    //    banner("JSONIFY-ing".to_string(), filename.clone());
    //    let mut normal_form = NormalForm::init();
    //
    //    println!("Generating Composition \n");
    //    composition.apply_to_normal_form(&mut normal_form);
    //
    ////    let e = event_from_init(init);
    //
    //    let norm_ev = generate_events(normal_form.operations, e);
    //
    //    write_composition_to_json(norm_ev, &filename).expect("Writing to JSON failed");
    //    printed("JSON".to_string());
}

fn create_pb_instance(n: usize) -> Arc<Mutex<ProgressBar<std::io::Stdout>>> {
    let mut pb = ProgressBar::new(n as u64);
    pb.format("╢w♬░╟");
    pb.message("Rendering:  ");
    Arc::new(Mutex::new(pb))
}

fn generate_waveforms(init: Init, mut norm_ev: Vec<Vec<PointOp>>) -> VecWav {
    println!("Rendering {:?} waveforms", norm_ev.len());
    let pb = create_pb_instance(norm_ev.len());

    let vec_wav = norm_ev
        .par_iter_mut()
        .map(|ref mut vec_point_op: &mut Vec<PointOp>| {
            pb.lock().unwrap().add(1 as u64);
            let mut osc = Oscillator::init(
                OscillatorBasis {
                    f: r_to_f64(init.f),
                    g: r_to_f64(init.g),
                    l: r_to_f64(init.l),
                    p: r_to_f64(init.p),
                },
                &get_default_app_settings(),
            );
            vec_point_op.render(&mut osc)
        })
        .collect();

    pb.lock().unwrap().finish_print(&"".to_string());

    vec_wav
}

fn sum_all_waveforms(mut vec_wav: VecWav) -> StereoWaveform {
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

fn sort_vecs(vec_wav: &mut VecWav) {
    vec_wav.sort_unstable_by(|a, b| b.l_buffer.len().cmp(&a.l_buffer.len()));
}

fn sum_vec(a: &mut Vec<f64>, b: &[f64]) {
    for (ai, bi) in a.iter_mut().zip(b) {
        *ai += *bi;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn render_equal() {
        let mut a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        sum_vec(&mut a, &b[..]);
        let expected = [2.0, 4.0, 6.0];
        assert_eq!(a, expected);
    }

    #[test]
    fn render_left() {
        let mut a = vec![1.0, 2.0, 3.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        sum_vec(&mut a, &b[..]);
        let expected = [2.0, 4.0, 6.0, 2.0];
        assert_eq!(a, expected);
    }
}
