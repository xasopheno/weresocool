use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Apply};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (4, 1, 0.0, 0.04, -0.5),
            (3, 1, 1.0, 0.04, 0.5),
            (5, 2, 5.0, 0.25, 1.0),
            (5, 2, 0.0, 0.25, 1.0),
            (3, 2, 4.0, 0.25, -1.0),
            (3, 2, 0.0, 0.25, -1.0),
            (1, 1, 0.0, 1.0, 0.0),
        ]
    }

    fn sequence1() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::Compose { operations: vec![
                    Op::Sequence { operations: vec![
                        Op::TransposeM { m: 7.0/4.0 },
                        Op::TransposeM { m: 3.0/2.0 },
                    ]},
                    Op::Gain {m: 0.5},
                    Op::Length { m: 0.5 }
                ]},
                Op::AsIs,
            ],
        }
    };

    fn sequence2() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::TransposeM { m: 7.0/8.0 },
                Op::TransposeM { m: 4.0/5.0 },
            ],
        }
    };

    fn sequence3() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::TransposeM { m: 4.0/3.0 },
                Op::TransposeM { m: 7.0/8.0 },
                Op::TransposeM { m: 6.0/5.0 },
            ],
        }
    };

    fn with_overtones() -> Op {
        Op::Compose {
            operations: vec![
                overtones(),
                sequence1(),
            ],
        }
    }

    fn our_thing() -> Op {
        Op::Compose {
            operations: vec![
                sequence1(),
                sequence2(),
                sequence3(),
            ],
        }
    }

    fn fit_our_thing_in_our_thing() -> Op {
        Op::Fit {
            n: 2,
            with_length_of: Box::new(our_thing()),
            main:
            Box::new(
            Op::Compose { operations: vec![
                    with_overtones(),
                    our_thing(),
                    Op::TransposeM { m: 3.0/1.0 },
                    Op::Gain { m: 0.25 }
               ]
            })
        }
    }

    fn fit_again() -> Op {
        Op::Fit {
            n: 2,
            with_length_of: Box::new(our_thing()),
            main:
            Box::new(
                Op::Compose { operations: vec![
                    fit_our_thing_in_our_thing(),
                    Op::TransposeM { m: 3.0/2.0 },
                    Op::Gain { m: 0.25 }
                ]
            })
        }
    }


    fn overlay() -> Op {
        Op::Overlay { operations: vec![
            fit_again(),
            fit_our_thing_in_our_thing(),
            our_thing(),
        ]}
    }

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(100.0, 1.0, 0.0, 1.5)];
    let mut events = overlay().apply(e);

    events.render(&mut oscillator)
}
