extern crate itertools;
extern crate num_rational;
extern crate pbr;
extern crate rayon;
extern crate socool_parser;
use event::{Event, Render};
use instrument::{
    oscillator::Oscillator,
    stereo_waveform::{Normalize, StereoWaveform},
};
use itertools::Itertools;
use num_rational::Rational64;
use operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use pbr::ProgressBar;
use rayon::prelude::*;
use settings::get_default_app_settings;
use socool_parser::{ast::Op, parser::Init};
use std::sync::{Arc, Mutex};
use ui::{banner, printed};
use write::{write_composition_to_json, write_composition_to_wav};

type PointOpSequences = Vec<Vec<PointOp>>;
type NormEv = Vec<Vec<Event>>;
type VecWav = Vec<StereoWaveform>;

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

pub fn event_from_init(init: Init) -> Event {
    Event::init(
        r_to_f64(init.f),
        r_to_f64(init.g),
        r_to_f64(init.p),
        r_to_f64(init.l),
    )
}

pub fn render(composition: &Op, init: Init) -> StereoWaveform {
    let mut normal_form = NormalForm::init();

    println!("\nGenerating Composition ");
    composition.apply_to_normal_form(&mut normal_form);

    let e = event_from_init(init);

    let norm_ev = generate_events(normal_form.operations, e);
    let vec_wav = generate_waveforms(norm_ev);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    result
}

pub fn render_mic(composition: &Op, event: Event) -> StereoWaveform {
    let mut normal_form = NormalForm::init();

    composition.apply_to_normal_form(&mut normal_form);
    let norm_ev = generate_events(normal_form.operations, event);
    let vec_wav = generate_waveforms(norm_ev);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    result
}

pub fn to_wav(composition: StereoWaveform, filename: String) {
    banner("Printing".to_string(), filename);
    write_composition_to_wav(composition);
    printed("WAV".to_string());
}

pub fn to_json(composition: &Op, init: Init, filename: String) {
    banner("JSONIFY-ing".to_string(), filename.clone());
    let mut normal_form = NormalForm::init();

    println!("Generating Composition \n");
    composition.apply_to_normal_form(&mut normal_form);

    let e = event_from_init(init);

    let norm_ev = generate_events(normal_form.operations, e);

    write_composition_to_json(norm_ev, &filename).expect("Writing to JSON failed");
    printed("JSON".to_string());
}

fn generate_events(sequences: PointOpSequences, event: Event) -> NormEv {
    let mut norm_ev: NormEv = vec![];
    for sequence in sequences {
        let mut event_sequence = vec![];
        for point_op in sequence {
            //            println!("{:?}", point_op);
            let mut e = event.clone();
            for mut sound in e.sounds.iter_mut() {
                sound.frequency *= r_to_f64(point_op.fm);
                sound.frequency += r_to_f64(point_op.fa);
                sound.pan *= r_to_f64(point_op.pm);
                sound.pan += r_to_f64(point_op.pa);
                sound.gain *= r_to_f64(point_op.g);
            }

            e.length *= r_to_f64(point_op.l);
            event_sequence.push(e)
        }
        norm_ev.push(event_sequence)
    }

    norm_ev
}

fn create_pb_instance(n: usize) -> Arc<Mutex<ProgressBar<std::io::Stdout>>> {
    let mut pb = ProgressBar::new(n as u64);
    pb.format("╢w♬░╟");
    pb.message("Rendering:  ");
    let pb = Arc::new(Mutex::new(pb));
    pb
}

fn generate_waveforms(mut norm_ev: NormEv) -> VecWav {
    println!("Rendering {:?} waveforms", norm_ev.len());
    let pb = create_pb_instance(norm_ev.len());

    let vec_wav = norm_ev
        .par_iter_mut()
        .map(|ref mut vec_events: &mut Vec<Event>| {
            pb.lock().unwrap().add(1 as u64);
            let mut osc = Oscillator::init(&get_default_app_settings());
            vec_events.render(&mut osc)
        })
        .collect();

    let finish_string = "".to_string();
    pb.lock().unwrap().finish_print(&finish_string);

    vec_wav
}

fn sum_all_waveforms(vec_wav: VecWav) -> StereoWaveform {
    let mut result = StereoWaveform::new(0);
    for wav in vec_wav {
        result.l_buffer = sum_vec(&result.l_buffer, wav.l_buffer);
        result.r_buffer = sum_vec(&result.r_buffer, wav.r_buffer)
    }

    result
}

fn sum_vec(a: &Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let vec_len = std::cmp::max(a.len(), b.len());
    let mut acc: Vec<f64> = vec![0.0; vec_len];
    for (i, e) in a.iter().zip_longest(&b).enumerate() {
        match e {
            itertools::EitherOrBoth::Both(v1, v2) => acc[i] = v1 + v2,
            itertools::EitherOrBoth::Left(e) => acc[i] = *e,
            itertools::EitherOrBoth::Right(e) => acc[i] = *e,
        }
    }

    acc
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn render_equal() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = sum_vec(&a, b);
        let expected = [2.0, 4.0, 6.0];
        assert_eq!(result, expected);
    }

    #[test]
    fn render_left() {
        let a = vec![1.0, 2.0, 3.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = sum_vec(&a, b);
        let expected = [2.0, 4.0, 6.0, 2.0];
        assert_eq!(result, expected);
    }

    #[test]
    fn render_right() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0, 1.0];
        let result = sum_vec(&a, b);
        let expected = [2.0, 4.0, 6.0, 1.0];
        assert_eq!(result, expected);
    }
}
