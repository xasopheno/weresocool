use event::{Event, Render};
use instrument::{Oscillator, StereoWaveform};
use operations::{Apply, Op};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (1, 1, 0.0, 1.0, -1.0),
            (1, 1, 3.0, 1.0, -1.0),
            (3, 1, 3.0, 0.25, -0.7),
            (3, 1, 0.0, 0.25, -0.7),
            (4, 1, 0.0, 0.44, -0.5),
            (4, 1, 5.0, 0.44, -0.5),
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
        Op::Compose {
            operations: vec![
                Op::Sequence {
                    operations: vec![
                        Op::Silence { m: 1.0 },
                        r![
                            (9, 8, 4.0, 0.1, -1.0),
                            (9, 8, 0.0, 0.1, -1.0),
                            (2, 1, 0.0, 0.2, -1.0),
                            (2, 1, 0.0, 0.2, -1.0),
                            (8, 5, 3.0, 0.2, 0.5),
                            (8, 5, 3.0, 0.2, 0.5),
                        ],
                        Op::Silence { m: 0.1 },
                        Op::Compose {
                            operations: vec![
                                r![(9, 1, 4.0, 0.5, 0.2), (9, 1, 0.0, 0.5, -0.2),],
                                Op::Gain { m: 0.5 },
                                Op::Length { m: 0.2 },
                            ],
                        },
                        Op::Silence { m: 0.1 },
                        r![
                            (5, 2, 5.0, 0.2, 1.0),
                            (5, 2, 0.0, 0.2, 1.0),
                            (9, 4, 0.0, 0.2, 1.0),
                            (9, 4, 5.0, 0.2, 1.0),
                            (3, 2, 3.0, 0.2, -0.5),
                            (3, 2, 0.0, 0.2, -0.5),
                        ],
                        Op::Silence { m: 1.0 },
                    ],
                },
                Op::TransposeM { m: 2.0 },
            ],
        }
    }

    fn repeat() -> Op {
        Op::Repeat {
            operations: vec![
                Op::AsIs,
                Op::TransposeM { m: 3.0 / 2.0 },
                Op::TransposeM { m: 8.0 / 5.0 },
                Op::TransposeM { m: 3.0 / 2.0 },
                Op::TransposeM { m: 4.0 / 3.0 },
                Op::TransposeM { m: 9.0 / 8.0 },
                Op::TransposeM { m: 7.0 / 4.0 },
                Op::TransposeM { m: 5.0 / 4.0 },
                Op::TransposeM { m: 4.0 / 3.0 },
                Op::TransposeM { m: 11.0 / 8.0 },
                Op::TransposeM { m: 13.0 / 8.0 },
                Op::TransposeM { m: 8.0 / 5.0 },
                Op::TransposeM { m: 5.0 / 3.0 },
                Op::TransposeM { m: 7.0 / 4.0 },
                Op::TransposeM { m: 15.0 / 8.0 },
                Op::TransposeM { m: 3.0 / 2.0 },
            ],
            n: 8,
        }
    }

    fn sequence1() -> Op {
        //        Op::Compose {
        //            operations: vec![
        //                repeat(),
        //        ]}
        Op::ComposeWithOrder {
            operations: vec![repeat(), Op::PanM { m: 0.85 }],
            order_fn,
        }
    };

    fn main() -> Op {
        Op::Compose {
            operations: vec![overtones(), sequence1()],
        }
    }
    let new_fit = Op::Fit {
        with_length_of: Box::new(main()),
        main: Box::new(Op::Compose {
            operations: vec![
                repeat(),
                r![
                    (14, 1, 0.0, 0.5, -1.0),
                    (14, 1, 0.0, 0.5, 1.0),
                    (13, 1, 0.0, 0.5, 1.0),
                    (13, 1, 0.0, 0.5, -1.0),
                    (12, 1, 0.0, 0.25, -1.0),
                    (12, 1, 0.0, 0.25, 1.0),
                    (11, 1, 0.0, 0.5, -1.0),
                    (10, 1, 0.0, 0.5, 1.0),
                    (9, 1, 0.0, 0.5, 1.0),
                    (8, 1, 0.0, 0.5, 1.0),
                    (7, 1, 0.0, 0.5, -1.0),
                    (5, 1, 0.0, 0.5, 1.0),
                    (6, 1, 0.0, 0.5, -1.0),
                ],
                Op::Gain { m: 0.08 },
                Op::Reverse {},
            ],
        }),
        n: 6,
    };

    let overlay = Op::Overlay {
        operations: vec![
            main(),
            new_fit,
            Op::Fit {
                with_length_of: Box::new(main()),
                main: Box::new(melody()),
                n: 6,
            },
        ],
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(80.0, 1.0, 0.0, 0.8)];
    let mut events = overlay.apply(e);

    events.render(&mut oscillator)
}
