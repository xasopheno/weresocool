use crate::instrument::{
    oscillator::{Basis, Oscillator},
    stereo_waveform::{Normalize, StereoWaveform},
};
use crate::render::{Render, RenderPointOp};
use crate::settings::default_settings;
use crate::ui::{banner, printed};
use crate::write::{write_composition_to_json, write_composition_to_wav};
use num_rational::Rational64;
use pbr::ProgressBar;
use rayon::prelude::*;
use serde_json::to_string;
use socool_ast::ast::OpOrNfTable;
use socool_ast::operations::{NormalForm, Normalize as NormalizeOp, PointOp};
use std::sync::{Arc, Mutex};
use crate::generation::video_data_generation::{
    vec_timed_op_to_vec_op4d, composition_to_vec_timed_op,
};

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

pub fn to_json(basis: &Basis, composition: &NormalForm, table: &OpOrNfTable, filename: String) {
    banner("JSONIFY-ing".to_string(), filename.clone());

    let vec_timed_op = composition_to_vec_timed_op(composition, table);
    let vec_op4d = vec_timed_op_to_vec_op4d(vec_timed_op, basis);

    let json = to_string(&vec_op4d).unwrap();

    write_composition_to_json(&json, &filename).expect("Writing to JSON failed");
    printed("JSON".to_string());
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

fn sum_vec(a: &mut Vec<f64>, b: &[f64]) {
    for (ai, bi) in a.iter_mut().zip(b) {
        *ai += *bi;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use socool_ast::ast::{Op::*, OpOrNf::*};

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

    #[test]
    fn to_vec_timed_op_test() {
        let mut normal_form = NormalForm::init();
        let pt = OpOrNfTable::new();

        Overlay {
            operations: vec![
                Op(Sequence {
                    operations: vec![
                        Op(PanA {
                            a: Rational64::new(1, 2),
                        }),
                        Op(TransposeM {
                            m: Rational64::new(2, 1),
                        }),
                        Op(Gain {
                            m: Rational64::new(1, 2),
                        }),
                        Op(Length {
                            m: Rational64::new(2, 1),
                        }),
                    ],
                }),
                Op(Sequence {
                    operations: vec![Op(Length {
                        m: Rational64::new(5, 1),
                    })],
                }),
            ],
        }
        .apply_to_normal_form(&mut normal_form, &pt);

        let timed_ops = composition_to_vec_timed_op(&normal_form, &pt);

        let op = TimedOp {
            fm: Rational64::new(1, 1),
            fa: Rational64::new(0, 1),
            pm: Rational64::new(1, 1),
            pa: Rational64::new(0, 1),
            g: Rational64::new(1, 1),
            l: Rational64::new(1, 1),
            t: Rational64::new(0, 1),
            event_type: EventType::On,
            voice: 0,
            event: 0,
        };

        assert_eq!(
            timed_ops,
            vec![
                TimedOp {
                    pa: Rational64::new(1, 2),
                    event_type: EventType::On,
                    ..op
                },
                TimedOp {
                    event_type: EventType::On,
                    l: Rational64::new(5, 1),
                    voice: 1,
                    ..op
                },
                TimedOp {
                    pa: Rational64::new(1, 2),
                    t: Rational64::new(1, 1),
                    event_type: EventType::Off,
                    ..op
                },
                TimedOp {
                    fm: Rational64::new(2, 1),
                    t: Rational64::new(1, 1),
                    event_type: EventType::On,
                    event: 1,
                    ..op
                },
                TimedOp {
                    fm: Rational64::new(2, 1),
                    t: Rational64::new(2, 1),
                    event_type: EventType::Off,
                    event: 1,
                    ..op
                },
                TimedOp {
                    g: Rational64::new(1, 2),
                    t: Rational64::new(2, 1),
                    event_type: EventType::On,
                    event: 2,
                    ..op
                },
                TimedOp {
                    g: Rational64::new(1, 2),
                    t: Rational64::new(3, 1),
                    event_type: EventType::Off,
                    event: 2,
                    ..op
                },
                TimedOp {
                    t: Rational64::new(3, 1),
                    l: Rational64::new(2, 1),
                    event_type: EventType::On,
                    event: 3,
                    ..op
                },
                TimedOp {
                    t: Rational64::new(5, 1),
                    l: Rational64::new(2, 1),
                    event_type: EventType::Off,
                    event: 3,
                    ..op
                },
                TimedOp {
                    t: Rational64::new(5, 1),
                    l: Rational64::new(5, 1),
                    event_type: EventType::Off,
                    voice: 1,
                    ..op
                },
            ]
        );
    }

    #[test]
    fn to_vec_op4d_test() {
        let basis = Basis {
            f: 100.0,
            g: 1.0,
            p: 0.0,
            l: 1.0,
            a: 44100.0,
            d: 44100.0,
        };

        let op = TimedOp {
            fm: Rational64::new(2, 1),
            fa: Rational64::new(0, 1),
            pm: Rational64::new(1, 1),
            pa: Rational64::new(1, 2),
            g: Rational64::new(1, 2),
            t: Rational64::new(0, 1),
            l: Rational64::new(1, 1),
            event_type: EventType::On,
            voice: 0,
            event: 0,
        };

        let vec_timed_op = vec![
            TimedOp {
                event_type: EventType::On,
                l: Rational64::new(3, 2),
                ..op
            },
            TimedOp {
                event_type: EventType::Off,
                l: Rational64::new(3, 2),
                t: Rational64::new(3, 2),
                ..op
            },
        ];

        let result = vec_timed_op_to_vec_op4d(vec_timed_op, &basis);
        let expected = vec![
            Op4D {
                t: 0.0,
                l: 1.5,
                event_type: EventType::On,
                voice: 0,
                event: 0,
                x: 0.5,
                y: 200.0,
                z: 0.5,
            },
            Op4D {
                t: 1.5,
                l: 1.5,
                event_type: EventType::Off,
                voice: 0,
                event: 0,
                x: 0.5,
                y: 200.0,
                z: 0.5,
            },
        ];
        assert_eq!(result, expected);
    }
}
