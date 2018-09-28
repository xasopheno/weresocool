use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;

fn composition() -> Op {
    fn overtones() -> Op {
        r![
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, -0.5),
            (1, 1, 3.0, 1.0, 0.5),
        ]
    }

    fn fasts() -> Op {
        compose![
            r![
                (3, 2, 3.0, 1.0, 0.0),
                (1, 1, 0.0, 1.0, -0.5),
                (1, 1, 7.0, 1.0, 0.5),
            ],
            sequence![
                TransposeM { m: 1.0 },
                TransposeM { m: 9.0/4.0 },
                TransposeM { m: 3.0/2.0 },
                TransposeM { m: 15.0/8.0 },
                TransposeM { m: 2.0/1.0 },
            ],
        ]
    }

    fn fasts_pan() -> Op {
        compose![
                fasts(),
                sequence![
                    PanA {a: 0.2},
                    PanA {a: -0.2},
                ],
            ]
    }

    fn fasts_fit() -> Op {
        fit![
            compose![
                sequence![
                    Silence {m: 1.0},
                    AsIs,
                ],
                fasts_pan(),
            ] =>
                compose![
                    sequence1(),
                    Length {m: 0.345}
                ], 1
        ]
    }

    fn sequence1() -> Op {
        sequence![
            r![
                (2, 1, 0.0, 1.0, 0.0),
                (2, 1, 4.0, 1.0, 0.0),
                (3, 2, 0.0, 1.0, -0.5),
                (3, 2, 3.0, 1.0, 0.5),
                (3, 2, 3.0, 1.0, 0.5),
                (3, 2, 3.0, 1.0, 0.5),
                (1, 1, 0.0, 1.0, -0.5),
                (1, 1, 3.0, 1.0, 0.5),
                (1, 2, 3.0, 1.0, 0.0),
            ],
            r![
                (2, 1, 0.0, 1.0, 0.0),
                (2, 1, 4.0, 1.0, 0.0),
                (9, 4, 0.0, 1.0, -0.5),
                (9, 4, 3.0, 1.0, 0.5),
                (5, 3, 3.0, 1.0, 0.5),
                (5, 3, 3.0, 1.0, 0.5),
                (9, 8, 0.0, 1.0, -0.5),
                (9, 8, 3.0, 1.0, 0.5),
                (1, 2, 3.0, 1.0, 0.0),
            ],
            compose![
                r![
                    (3, 1, 0.0, 1.0, 0.0),
                    (3, 1, 4.0, 1.0, 0.0),
                    (15, 8, 0.0, 1.0, -0.5),
                    (15, 8, 3.0, 1.0, 0.5),
                    (2, 1, 3.0, 1.0, 0.5),
                    (2, 1, 3.0, 1.0, 0.5),
                    (5, 4, 0.0, 1.0, -0.5),
                    (5, 4, 3.0, 1.0, 0.5),
                    (1, 2, 3.0, 1.0, 0.0),
                ],
                Length {m: 2.0}
            ]
        ]
    }

    overlay![
        compose![
            fasts_fit(),
            sequence![
              Silence {m: 1.0},
              AsIs,
              AsIs,
            ],
            Gain {m: 6.0}
        ],
        compose![
            overtones(),
            sequence1()
        ]
    ]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(275.0, 0.75, 0.0, 1.2)
}

fn generate_events(event: Event, operation: fn() -> Op) -> Vec<Event> {
    operation().apply(vec![event])
}

pub fn operations() -> Op {
    composition()
}

pub fn events() -> Vec<Event> {
    generate_events(event(), composition)
}

pub fn generate_composition() -> StereoWaveform {
    events().render(&mut oscillator())
}
