use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let rs = r![
        (1, 1, 0.0, 0.6, -0.3),
        (1, 1, 0.0, 0.6, 0.3),
        (3, 2, -3.0, 1.0, 0.0),
        (3, 2, 2.0, 1.0, 0.0),
    ];

    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Compose {
                operations: vec![
                    Op::AsIs,
                    Op::Ratios {
                        ratios: r![
                            (1, 1, 0.0, 0.6, 0.3),
                            (1, 1, 0.0, 0.6, -0.3),
                            (3, 2, 3.0, 1.0, 1.0),
                            (3, 2, -2.0, 1.0, -1.0),
                        ],
                    },
                ],
            },
            Op::Transpose {
                m: 3.0 / 2.0,
                a: 0.0,
            },
            Op::Compose {
                operations: vec![
                    Op::Transpose {
                        m: 9.0 / 2.0,
                        a: 0.0,
                    },
                    Op::Gain { m: 0.5, a: 0.0 },
                ],
            },
            Op::Compose {
                operations: vec![
                    Op::Transpose {
                        m: 15.0 / 2.0,
                        a: 0.0,
                    },
                    Op::Gain { m: 0.55, a: 0.0 },
                    Op::Ratios {
                        ratios: r![
                            (1, 1, 2.0, 1.0, 0.0),
                            (3, 2, -3.0, 1.0, -1.0),
                            (5, 4, 2.0, 1.0, 1.0),
                            (15, 8, 0.0, 1.0, 0.0),
                        ],
                    },
                ],
            },
        ],
    };

    let sequence2 = Op::Sequence {
        operations: vec![
            Op::Transpose {
                m: 9.0 / 8.0,
                a: 0.0,
            },
            Op::Compose {
                operations: vec![
                    Op::AsIs,
                    Op::Ratios {
                        ratios: r![
                            (1, 1, 2.0, 1.0, 0.2),
                            (7, 4, 2.0, 1.0, -0.2),
                            (16, 5, -3.0, 0.7, 1.0),
                            (5, 2, 0.0, 1.0, -1.0),
                        ],
                    },
                ],
            },
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 15.0 / 16.0,
                a: 0.0,
            },
        ],
    };

    let sequence3 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose {
                m: 2.0 / 3.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 9.0 / 8.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 3.0 / 4.0,
                a: 0.0,
            },
        ],
    };

    let ops3 = Op::Compose {
        operations: vec![
            sequence1,
            sequence2,
            sequence3,
            Op::Compose {
                operations: vec![
                    Op::Gain { m: 0.5, a: 0.0 },
                    Op::Sequence {
                        operations: vec![
                            Op::AsIs,
                            Op::AsIs,
                            Op::AsIs,
                            Op::AsIs,
                            Op::AsIs,
                            Op::AsIs,
                        ],
                    },
                ],
            },
        ],
    };

    let mut oscillator = Oscillator::init(rs.clone(), &get_default_app_settings());
    let e = vec![Event::new(100.0, rs.clone(), 0.11, 0.75)];
    let mut events = ops3.apply(e);
    events.render(&mut oscillator)
}
