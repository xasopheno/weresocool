use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose { m: 2.0, a: 0.0 },
            Op::Length { m: 0.5 },
            Op::Silence { m: 1.0 },
            Op::Transpose { m: 1.5, a: 0.0 },
            Op::Gain { m: 0.5 },
            Op::Transpose {
                m: 5.0 / 4.0,
                a: 0.0,
            },
            Op::AsIs,
        ],
    };

    let sequence2 = Op::Fit {
        with_length_of: Box::new(sequence1.clone()),
        main: Box::new(Op::Sequence {
            operations: vec![sequence1.clone(), sequence1.clone()],
        }),
    };

    let sequence3 = Op::Sequence {
        operations: vec![sequence1.clone(), sequence2.clone()],
    };

    let sequence4 = Op::Compose {
        operations: vec![
            sequence3.clone(),
            Op::Length { m: 0.5 },
            Op::Sequence {
                operations: vec![
                    Op::AsIs,
                    Op::Transpose {
                        m: 9.0 / 8.0,
                        a: 0.0,
                    },
                    Op::Transpose {
                        m: 6.0 / 5.0,
                        a: 0.0,
                    },
                ],
            },
            Op::Sequence {
                operations: vec![
                    Op::AsIs,
                    Op::Transpose {
                        m: 9.0 / 8.0,
                        a: 0.0,
                    },
                    Op::Transpose {
                        m: 6.0 / 5.0,
                        a: 0.0,
                    },
                ],
            },
        ],
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(300.0, 1.0, 0.0, 0.5)];
    let mut events = sequence4.apply(e);

    events.render(&mut oscillator)
}
