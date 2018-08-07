use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let rs = r![
        (0, 1, 0.0, 1.0, 0.0),
        (0, 1, 0.0, 1.0, 0.0),
        (0, 1, 3.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
        //
        (0, 1, 2.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
        //
        (0, 1, 2.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
        //
        (0, 1, 2.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
        (0, 1, 2.0, 1.0, 0.0),
    ];

    fn bend(distance: usize) -> Op {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        fn bend_ratios(distance:usize) -> Vec<R> {
            r![
                (2, 3, 0.0-(distance as f32) * 0.075, 0.25, -0.5 + (distance as f32 / 200.0)),
                (2, 3, 0.0+(distance as f32) * 0.0, 0.25, 0.5 - (distance as f32 / 200.0)),
                (5, 1, 10.0, 0.55, 0.3),
                (5, 1, 0.0, 0.55, -0.3),
//
                (2, 1, 0.0+(distance as f32) * 0.1, 0.3, -1.0 + (distance as f32 / 50.0)),
                (2, 1, 0.0+(distance as f32) * 0.15, 0.3, -1.0 + (distance as f32 / 50.0)),
                (2, 1, 5.0+(distance as f32) * 0.2, 0.3, 1.0 - (distance as f32 / 50.0)),
//
                (3, 1, 10.0-distance as f32, 0.4, -1.0 + (distance as f32 / 50.0)),
                (3, 1, 0.0-distance as f32, 0.3, 1.0 - (distance as f32 / 50.0)),
                (3, 1, 5.0-distance as f32, 0.3, -1.0 + (distance as f32 / 50.0)),
//
                (4, 1, 0.0-distance as f32, 0.3, 1.0 - (distance as f32 / 50.0)),
                (4, 1, 10.0-(distance as f32) * 1.0, 0.15, 1.0 - (distance as f32 / 50.0)),
                (4, 1, 0.0-(distance as f32) * 1.0, 0.15, -1.0 + (distance as f32 / 50.0)),
            ]
        }

        let mut ops: Vec<Op> = vec![];

        ops.push(Op::Compose {
            operations: vec![
                Op::Length { m: 15.0, a: 0.0 },
                Op::Ratios {
                    ratios: bend_ratios(0),
                },
            ],
        });

        for i in 0..distance {
            ops.push(Op::Ratios {
                ratios: bend_ratios(i),
            })
        }

        Op::Sequence { operations: ops }
    }

    let sequence1 = Op::Sequence {
        operations: vec![bend(125)],
    };

    let sequence2 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::AsIs,
            Op::AsIs,
            Op::AsIs,
            Op::Transpose {
                m: 5.0 / 4.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 9.0 / 8.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 5.0 / 4.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
        ],
    };

    let sequence3 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
            Op::AsIs,
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
            Op::AsIs,
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
            Op::AsIs,
            Op::AsIs,
        ],
    };

    let sequence3 = Op::Compose {
        operations: vec![sequence1.clone(), sequence2.clone(), sequence3.clone()],
    };

    let mut oscillator = Oscillator::init(rs.clone(), &get_default_app_settings());
    let e = vec![Event::new(135.0, rs.clone(), 0.013, 0.5)];
    let mut events = sequence3.apply(e);

    events.render(&mut oscillator)
}
