use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;
use std::fs::File;
use std::io::prelude::*;


pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
//            (6, 1, 0.0, 0.14, 0.0),
//            (5, 1, 2.0, 0.24, 0.0),
//            (5, 1, 0.0, 0.24, 0.0),
//            (3, 1, 1.0, 0.14, 0.5),
//            (3, 1, 0.0, 0.14, 0.5),
//            (2, 1, 3.0, 0.5, 0.0),
//            (2, 1, 0.0, 0.5, 0.0),
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, -0.5),
            (1, 1, 3.0, 1.0, 0.5),
        ]
    }

    fn sequence1() -> Op {
        sequence![
            Op::AsIs,
            Op::TransposeM { m: 5.0 / 3.0 },
            Op::TransposeM { m: 3.0 / 2.0 },
            Op::TransposeM { m: 9.0 / 4.0 },
            Op::AsIs,
        ]
    };


    fn sequence2() -> Op {
        sequence![
            Op::TransposeM { m: 1.0 / 1.0 },
            Op::TransposeM { m: 1.0 / 1.0 },
            Op::TransposeM { m: 3.0 / 4.0 },
            Op::TransposeM { m: 2.0 / 3.0 },
        ]
    };

    fn sequence3() -> Op {
        sequence![
            Op::TransposeM { m: 1.0 / 1.0 },
            Op::TransposeM { m: 1.0 / 1.0 },
            //            Op::TransposeM { m: 3.0/2.0 },
            //            Op::TransposeM { m: 3.0/2.0 },
        ]
    };

    fn fit() -> Op {
        Op::Fit {
            n: 6,
            with_length_of: Box::new(main()),
            main: Box::new(compose![
                main(),
                TransposeM { m: 3.0 / 2.0 },
                Gain { m: 0.3 },
            ]
        ),
        }
    }

    fn fit_again() -> Op {
        Op::Fit {
            n: 12,
            with_length_of: Box::new(main()),
            main: Box::new(compose![
                main(),
                TransposeM { m: 9.0 / 4.0 },
                Gain { m: 0.3 },
//                Reverse {}
            ]),
        }
    }

    fn fit_again_again() -> Op {
        Op::Fit {
            n: 24,
            with_length_of: Box::new(main()),
            main: Box::new(compose![
                main(),
                TransposeM { m: 3.0 / 1.0 },
                Gain { m: 0.1 }
            ]),
        }
    }

    fn main() -> Op {
        compose![
//            overtones(),
            sequence1(),
            sequence2(),
            sequence3(),
            sequence2(),
        ]
    };

    fn chords() -> Op {
        sequence![
            r![
                (5, 4, 3.0, 1.0, 0.0),
                (9, 8, -5.0, 1.0, 0.0),
                (1, 1, 0.0, 1.0, 0.0),
            ],
            r![
                (11, 8, 0.0, 1.0, -0.2),
                (5, 4, 0.0, 1.0, 0.2),
                (9, 8, 0.0, 1.0, 0.0),
               ],
            r![
                (3, 2, 3.0, 1.0, -0.3),
                (3, 2, 0.0, 1.0, 0.3),
                (11, 8, 0.0, 1.0, 0.0),
            ],
            r![
                (15, 16, 0.0, 1.0, -0.4),
                (5, 4, 5.0, 1.0, 0.4),
                (9, 8, 0.0, 1.0, 0.0),
            ],
            compose![
                r![
                    (5, 3, 0.0, 1.0, -0.5),
                    (3, 2, 0.0, 1.0, 0.5),
                    (9, 8, 0.0, 1.0, -0.5),
                ],
                Gain { m: 0.75 }
            ],
            compose![
                r![
                    (3, 2, 0.0, 1.0, -0.4),
                    (3, 2, -4.0, 1.0, 0.4),
                    (11, 8, 0.0, 1.0, 0.0),
                ],
                Gain {m: 0.75}
            ],
            compose![
                r![
                    (5, 4, 3.0, 1.0, -0.2),
                    (5, 4, -7.0, 1.0, 0.2),
                    (5, 4, 0.0, 1.0, 0.0),
                ],
                Gain {m: 0.75}
            ],
            r![
                (9, 8, -6.0, 1.0, 0.0),
                (1, 1, 0.0, 1.0, 0.0),
                (15, 16, 4.0, 1.0, 0.0),
            ],
        ]
    }

    fn fit_chords() -> Op {
        Op::Fit {
            n: 10,
            with_length_of: Box::new(main()),
            main: Box::new(compose![
                overtones(),
                chords(),
                TransposeM { m: 1.0 / 1.0 },
                Gain { m: 0.4 }
            ]),
        }
    }

    fn overlay() -> Op {
        overlay![
            fit_chords(),
//            fit(),
            fit_again(),
            fit_again_again(),
//            main(),
//            compose![
//                main(),
//                TransposeM {m: 0.5},
//                Gain {m: 1.5},
//            ]
        ]
    }


    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(200.0, 1.0, 0.0, 1.8)];
    let mut events = overlay().apply(e);

//    println!("{:?}", events);

    events.render(&mut oscillator)
}
