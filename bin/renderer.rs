extern crate itertools;
extern crate weresocool;
extern crate portaudio;
extern crate socool_parser;
use portaudio as pa;
use socool_parser::{
    parser::*,
    ast::Op
};
use itertools::Itertools;
use weresocool::{
    event::{Event, Render},
    instrument::{
        oscillator::Oscillator,
        stereo_waveform::{StereoWaveform, Normalize}
    },
    generation::parsed_to_waveform::{event_from_init},
    operations::{Apply, GetOperations, Normalize as NormalizeOp},
    settings::get_default_app_settings,
    portaudio_setup::output::setup_portaudio_output,
    ui::{get_args, no_file_name, were_so_cool_logo},
    examples::documentation,
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

    let pa = pa::PortAudio::new()?;

    let mut output_stream = setup_portaudio_output(composition, &pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;

    Ok(())
}

fn render(composition: &NormOp, init: Init) -> StereoWaveform {
    let input = vec![vec![Op::AsIs]];
    let piece = composition
        .apply_to_normal_form(input);
//        .iter()
//        .flat_map(|array| array.iter())
//        .cloned()
//        .collect();

    println!("PIECE {:?}", piece);
    let mut seqs = vec![];
    for seq in piece {
        seqs.push(Op::Sequence { operations: seq})
    }

    let normal_form_op = Op::Overlay {
        operations: seqs

    };

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

fn generate_waveforms(norm_ev: NormEv) -> VecWav {
    let mut vec_wav: VecWav = vec![];
    for mut vec_events in norm_ev {
        let mut osc = Oscillator::init(&get_default_app_settings());
        vec_wav.push(vec_events.render(&mut osc))
    }

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
            itertools::EitherOrBoth::Right(e) => acc[i] = *e
        }
    }

    acc
}

//#[cfg(test)]
//pub mod tests {
//    use super::*;
//    #[test]
//    fn test_render() {}
//}
