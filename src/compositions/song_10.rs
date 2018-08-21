use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let macro_test = r![
        (1, 1, 0.0, 1.0, 0.0),
        (1, 1, 3.0, 1.0, 0.0),
        (3, 1, 1.0, 0.14, 0.5),
        (3, 1, 0.0, 0.14, 0.5),
        (2, 1, 0.0, 0.1, -0.5),
        (2, 1, 5.0, 0.1, -0.5),
    ];

    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::TransposeM { m: 2.0 },
            Op::Length { m: 0.5 },
            Op::Silence { m: 1.0 },
            Op::TransposeM { m: 1.5 },
            Op::Gain { m: 0.5 },
            Op::TransposeM { m: 5.0 / 4.0 },
            Op::Compose {
                operations: vec![
                    Op::Sequence {
                        operations: vec![Op::PanA { a: 0.5 }, Op::PanA { a: -0.5 }],
                    },
                    Op::Length { m: 0.5 },
                ],
            },
        ],
    };

    let sequence1 = Op::Compose {
        operations: vec![macro_test.clone(), sequence1],
    };

    let sequence2 = Op::Fit {
        n: 2,
        with_length_of: Box::new(sequence1.clone()),
        main: Box::new(sequence1.clone()),
    };

    let sequence3 = Op::Sequence {
        operations: vec![sequence1.clone(), sequence2.clone()],
    };

    let overlay = Op::Overlay {
        operations: vec![
            sequence1.clone(),
            sequence3.clone(),
            Op::Compose {
                operations: vec![sequence2.clone(), Op::TransposeM { m: 3.0 }],
            },
        ],
    };

    let overlay2 = Op::Overlay {
        operations: vec![
            overlay.clone(),
            //            sequence3.clone(),
            Op::Compose {
                operations: vec![sequence2.clone(), Op::TransposeM { m: 3.0 }],
            },
        ],
    };

    let sequence4 = Op::Compose {
        operations: vec![
            overlay.clone(),
            Op::Length { m: 0.5 },
            Op::Sequence {
                operations: vec![
                    Op::AsIs,
                    Op::TransposeM { m: 9.0 / 8.0 },
                    Op::TransposeM { m: 5.0 / 4.0 },
                ],
            },
            Op::Sequence {
                operations: vec![
                    Op::AsIs,
                    Op::TransposeM { m: 9.0 / 8.0 },
                    Op::TransposeM { m: 6.0 / 5.0 },
                ],
            },
        ],
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(120.0, 0.35, 0.0, 0.8)];
    let mut events = sequence4.apply(e);

    events.render(&mut oscillator)
}
