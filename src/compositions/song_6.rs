use ratios::{R, Pan};
use operations::{Op, Operate};
use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let rs = r![
        (1, 1, 0.0, 0.6, -0.3),
        (1, 1, 0.0, 0.6, 0.3),
        (3, 2, -3.0, 1.0, 0.0),
        (3, 2, 2.0, 1.0, 0.0),
    ];
    let e = vec![Event::new(100.0, rs.clone(), 0.11, 1.0)];

    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Compose { operations: vec![
                Op::AsIs,
                Op::Ratios { ratios:
                    r![
                            (1, 1, 0.0, 0.6, 0.3),
                            (1, 1, 0.0, 0.6, -0.3),
                            (3, 2, -3.0, 1.0, 1.0),
                            (3, 2, 2.0, 1.0, -1.0),
                        ]
                    }
                ]
            },
            Op::Transpose { m: 3.0/2.0, a: 0.0 },
            Op::Compose { operations: vec![
                Op::Transpose { m: 9.0/2.0, a: 0.0 },
                Op::Gain {m: 0.5, a: 0.0}
            ] },
            Op::Compose { operations: vec![
                Op::Transpose { m: 15.0/2.0, a: 0.0 },
                Op::Gain {m: 0.55, a: 0.0},
                Op::Ratios { ratios:
                    r![
                        (1, 1, 2.0, 1.0, 0.0),
                        (3, 2, -3.0, 1.0, -1.0),
                        (5, 4, 2.0, 1.0, 1.0),
                        (15, 8, 0.0, 1.0, 0.0),
                    ]
                }
            ]},
        ]
    };

    let sequence2 = Op::Sequence {
        operations: vec![
            Op::Transpose { m: 9.0/8.0, a: 0.0 },
            Op::AsIs,
            Op::Transpose { m: 4.0/3.0, a: 0.0 },
            Op::Transpose { m: 15.0/16.0, a: 0.0 },

        ]
    };

    let sequence3 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose { m: 2.0/3.0, a: 0.0 },
            Op::Transpose { m: 9.0/8.0, a: 0.0 },
            Op::Transpose { m: 3.0/4.0, a: 0.0 },

        ]
    };

    let ops3 = Op::Compose {
        operations: vec![
            sequence1.clone(),
            sequence2.clone(),
            sequence3.clone(),
            Op::Sequence {
                operations: vec![
                    Op::AsIs,
                    Op::AsIs,
                    Op::AsIs,
                    Op::AsIs,
                    Op::AsIs,
                ]
            }
        ]
    };

//    println!("{:?}", ops.apply(e.clone()));
    let mut oscillator = Oscillator::init(rs.clone(), &get_default_app_settings());

    println!("{:?}", ops3.apply(e.clone()));
    let mut events = ops3.apply(e.clone());
    events.render(&mut oscillator)
}
