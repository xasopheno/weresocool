use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Apply};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (1, 1, 0.0, 1.0, -1.0),
            (1, 1, 3.0, 1.0, -1.0),
            (3, 1, 3.0, 0.14, -0.7),
            (3, 1, 0.0, 0.14, -0.7),
            (4, 1, 0.0, 0.4, -0.5),
            (4, 1, 5.0, 0.4, -0.5),
        ]
    }

    fn order_fn(order: usize, _length: usize) -> f32 {
        if order % 3 == 0 || order % 4 == 0 {
            1.0
        } else {
            -1.0
        }
    }

    fn melody() -> Op {
        Op::Compose { operations: vec![
            Op::Sequence { operations: vec![
                Op::Silence { m: 1.0 },
                r![
                    (2, 1, 0.0, 0.25, -1.0),
                    (8, 5, 3.0, 0.25, 0.5),
                ],
                r![
                    (9, 4, 0.0, 0.25, 1.0),
                    (9, 4, 5.0, 0.25, 1.0),
                    (1, 1, 3.0, 0.25, -0.5),
                ],
                Op::Silence { m: 1.0 },
            ]},
            Op::TransposeM { m: 2.0 }
        ]}
    }

    fn repeat() -> Op {
        Op::Repeat { operations: vec![
            Op::AsIs,
            Op::TransposeM { m: 3.0/2.0 },
            Op::TransposeM { m: 8.0/5.0 },
            Op::TransposeM { m: 3.0/2.0 },

            Op::TransposeM { m: 4.0/3.0 },
            Op::TransposeM { m: 9.0/8.0 },
            Op::TransposeM { m: 7.0/4.0 },
            Op::TransposeM { m: 5.0/4.0 },

            Op::TransposeM { m: 4.0/3.0 },
            Op::TransposeM { m: 11.0/8.0 },
            Op::TransposeM { m: 3.0/2.0 },
            Op::TransposeM { m: 8.0/5.0 },

            Op::TransposeM { m: 5.0/3.0 },
            Op::TransposeM { m: 7.0/4.0 },
            Op::TransposeM { m: 15.0/8.0 },
            Op::TransposeM { m: 3.0/2.0 },
        ],
            n: 8
        }
    }

    fn sequence1() -> Op {
//        Op::Compose {
//            operations: vec![
//                repeat(),
//        ]}
        Op::ComposeWithOrder {
            operations: vec![
                repeat(),
                Op::PanM { m: 0.85 }
            ],
            order_fn
        }
    };



    let main = Op::Compose {
        operations: vec![overtones(), sequence1()],
    };

    let new_fit = Op::Fit {
        with_length_of: Box::new(main.clone()),
        main: Box::new(
            Op::Compose {operations: vec![
                repeat(),
                r![
                    (8, 1, 0.0, 0.5, 1.0),
                    (7, 1, 0.0, 0.5, -1.0),
                    (5, 1, 0.0, 0.5, 1.0),
                    (6, 1, 0.0, 0.5, -1.0),
                ],
                Op::Gain { m: 0.1 },
                Op::Reverse {},
            ]}
        ),
        n: 6,
    };

    let overlay = Op::Overlay {
        operations: vec![
            main.clone(),
            new_fit,
            Op::Fit {
                with_length_of: Box::new(main.clone()),
                main: Box::new(melody()),
                n: 6,
            }
        ]
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(80.0, 1.0, 0.0, 0.8)];
    let mut events = overlay.apply(e);

    events.render(&mut oscillator)
}
