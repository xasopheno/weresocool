use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, -0.5),
            (1, 1, 3.0, 1.0, 0.5),
        ]
    }

    fn sequence1() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 5.0 / 3.0 },
            TransposeM { m: 3.0 / 2.0 },
            TransposeM { m: 9.0 / 4.0 },
            AsIs,
        ]
    };


    fn sequence2() -> Op {
        sequence![
            TransposeM { m: 1.0 / 1.0 },
            TransposeM { m: 1.0 / 1.0 },
            TransposeM { m: 3.0 / 4.0 },
            TransposeM { m: 2.0 / 3.0 },
        ]
    };

    fn sequence3() -> Op {
        sequence![TransposeM { m: 1.0 / 1.0 }, TransposeM { m: 1.0 / 1.0 },]
    };

    fn depth_1() -> Op {
        fit! {
            compose![
                main(),
                TransposeM { m: 9.0 / 4.0 },
                Gain { m: 0.3 },
            ] => main(), 12
        }
    }

    fn depth_2() -> Op {
        fit! {
            compose![
                main(),
                TransposeM { m: 3.0 / 1.0 },
                Gain { m: 0.1 }
            ] => main(), 24
        }
    }

    fn main() -> Op {
        compose![sequence1(), sequence2(), sequence3(), sequence2(),]
    };

    fn chords() -> Op {
        sequence![
            r![
                (5, 4, 3.0, 1.0, 0.5),
                (9, 8, -5.0, 1.0, -0.5),
                (1, 1, 0.0, 1.0, 0.0),
            ],
            r![
                (11, 8, 0.0, 1.0, -0.2),
                (5, 4, 0.0, 1.0, 0.2),
                (9, 8, 0.0, 1.0, 0.0),
            ],
            r![
                (3, 2, 3.0, 1.0, -0.3),
                (11, 8, 0.0, 1.0, 0.0),
                (5, 4, 0.0, 1.0, 0.3),
            ],
            compose![r![
                (15, 16, 0.0, 1.0, -0.4),
                (5, 4, 5.0, 1.0, 0.4),
                (9, 8, 0.0, 1.0, 0.0),
            ],],
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
                Gain { m: 0.75 }
            ],
            compose![
                r![
                    (5, 4, 3.0, 1.0, -0.2),
                    (5, 4, -7.0, 1.0, 0.2),
                    (5, 4, 0.0, 1.0, 0.0),
                ],
                Gain { m: 0.75 },
            ],
            compose![
                r![
                    (9, 8, -6.0, 1.0, -0.4),
                    (15, 16, 4.0, 1.0, 0.4),
                    (3, 4, 0.0, 1.0, 0.0),
                ],
                Gain { m: 0.75 },
            ]
        ]
    }

    fn fit_chords() -> Op {
        fit! {
            compose![
                overtones(),
                chords(),
                Gain { m: 0.4 }
            ] => main(), 10
        }
    }

    fn overlay() -> Op {
        overlay![fit_chords(), depth_1(), depth_2(),]
    }

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(200.0, 0.25, 0.0, 1.8)];
    let mut events = overlay().apply(e);

    events.render(&mut oscillator)
}
