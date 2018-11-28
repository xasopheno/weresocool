extern crate itertools;
extern crate portaudio;
extern crate rayon;
extern crate socool_parser;
extern crate weresocool;
use itertools::Itertools;
use portaudio as pa;
use rayon::prelude::*;
use socool_parser::{ast::Op, parser::*};
use weresocool::{
    event::{Event, Render},
    examples::documentation,
    generation::parsed_to_waveform::event_from_init,
    instrument::{
        oscillator::Oscillator,
        stereo_waveform::{Normalize, StereoWaveform},
    },
    operations::{Apply, GetOperations, NormalForm, Normalize as NormalizeOp, PointOp},
    portaudio_setup::output::setup_portaudio_output,
    settings::get_default_app_settings,
    ui::{banner, get_args, no_file_name, printed, were_so_cool_logo},
    write::write_composition_to_wav,
};

type NormOp = Op;
type Sequences = Vec<Op>;
type NormEv = Vec<Vec<Event>>;
type VecWav = Vec<StereoWaveform>;

fn main() -> Result<(), pa::Error> {
    were_so_cool_logo();
    let args = get_args();

    if args.is_present("doc") {
        documentation();
    }

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let parsed = parse_file(&filename.unwrap().to_string());
    let main = parsed.table.get("main").unwrap();
    let init = parsed.init;

    let composition = render(main, init);

    if args.is_present("print") {
        banner("Printing".to_string(), filename.unwrap().to_string());
        write_composition_to_wav(composition);
        printed("WAV".to_string());
    } else {
        let pa = pa::PortAudio::new()?;

        let mut output_stream = setup_portaudio_output(composition, &pa)?;
        output_stream.start()?;

        while let true = output_stream.is_active()? {}

        output_stream.stop()?;
    }

    Ok(())
}

fn point_ops_to_ops(input: Vec<Vec<PointOp>>) -> Vec<Vec<Op>> {
    let mut result = vec![];
    for sequence in input {
        let mut seq = vec![];
        for point_op in sequence {
            seq.push(point_op.to_op())
        }
        result.push(seq);
    }

    result
}

fn render(composition: &NormOp, init: Init) -> StereoWaveform {
    let mut piece = NormalForm::init();

    println!("Applying Operations \n");

    composition.apply_to_normal_form(&mut piece);

    let mut piece = point_ops_to_ops(piece.operations);

    let voices = piece
        .iter_mut()
        .map(|voice| Op::Sequence {
            operations: voice.to_owned(),
        })
        .collect();

    let normal_form_op = Op::Overlay { operations: voices };

    let sequences: Sequences = normal_form_op.get_operations().expect("Not in Normal Form");

    let e = event_from_init(init);

    let norm_ev = generate_events(sequences, e);
    let vec_wav = generate_waveforms(norm_ev);
    let mut result = sum_all_waveforms(vec_wav);
    result.normalize();

    result
}

fn generate_events(sequences: Sequences, event: Event) -> NormEv {
    let mut norm_ev: NormEv = vec![];
    for sequence in sequences {
        norm_ev.push(sequence.apply(vec![event.clone()]))
    }

    norm_ev
}

fn generate_waveforms(mut norm_ev: NormEv) -> VecWav {
    println!("Generating {:?} waveforms", norm_ev.len());
    let vec_wav = norm_ev
        .par_iter_mut()
        .map(|ref mut vec_events: &mut Vec<Event>| {
            let mut osc = Oscillator::init(&get_default_app_settings());
            vec_events.render(&mut osc)
        })
        .collect();

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

fn sum_vec(a: &Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let vec_len = std::cmp::max(a.len(), b.len());
    let mut acc: Vec<f32> = vec![0.0; vec_len];
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

    fn render_left() {
        let a = vec![1.0, 2.0, 3.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = sum_vec(&a, b);
        let expected = [2.0, 4.0, 6.0, 2.0];
        assert_eq!(result, expected);
    }

    fn render_right() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0, 1.0];
        let result = sum_vec(&a, b);
        let expected = [2.0, 4.0, 6.0, 1.0];
        assert_eq!(result, expected);
    }
}
