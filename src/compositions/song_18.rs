use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;

fn composition() -> Op {
    fn overtones() -> Op {
        r![
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, -0.25),
            (1, 1, 3.0, 1.0, -0.25),
        ]
    }

    fn sequence1() -> Op {
        sequence![
            TransposeM { m: 3.0 / 2.0 },
            TransposeM { m: 3.0 / 2.0 },
            AsIs,
            AsIs,
            TransposeM { m: 3.0 / 2.0 },
            AsIs,
        ]
    }

    fn bass() -> Op {
        compose![
            r![
                (8, 1, 0.0, 0.2, 0.0),
                (1, 1, 0.0, 1.0, -0.25),
                (1, 1, 5.0, 1.0, 0.25),
            ],
            sequence![
                Silence {m: 0.25},
                compose![
                    r![
                        (11, 1, 11.0, 0.05, -0.75),
                        (11, 1, 0.0, 0.05, 0.75),
                        (8, 1, 7.0, 0.15, 1.0),
                        (8, 1, 0.0, 0.15, -1.0),
                        (1, 1, 0.0, 0.15, -0.25),
                        (1, 1, 3.0, 1.0, 0.25),
                        (1, 2, 0.0, 0.5, -1.0),
                        (1, 2, 1.0, 0.5, 1.0),
                    ],
                    TransposeM {m: 4.0/5.0},
                ],
                Silence {m: 1.25},
                compose![
                    sequence![
                        TransposeM {m: 5.0/2.0},
                        TransposeM {m: 9.0/4.0},
                        TransposeM {m: 3.0/1.0},
                        TransposeM {m: 9.0/4.0},
                        TransposeM {m: 2.0/1.0},
                        TransposeM {m: 8.0/3.0},
                        TransposeM {m: 7.0/4.0},
                        TransposeM {m: 2.0/1.0},
                        compose![
                            sequence![
                                compose![
                                    r![
                                        (1, 1, -5.0, 0.333, 1.0),
                                        (1, 1, 5.0, 0.333, -1.0),
                                        (1, 1, 0.0, 0.333, 1.0),
                                    ],
                                    sequence![
                                        TransposeM {m: 5.0/3.0},
                                        TransposeM {m: 10.0/3.0},
                                        TransposeM {m: 20.0/3.0},
                                        TransposeM {m: 40.0/3.0},
                                        TransposeM {m: 60.0/3.0},
                                        TransposeM {m: 80.0/3.0},
                                        TransposeM {m: 100.0/3.0},
                                    ],
                                    Gain {m: 0.4}
                                ]
                            ],
                            Length {m: 0.3}
                        ]
                    ],
                    Length {m: 1.8}
                ],
                compose![
                    sequence![
                        Silence {m: 0.25},
                        TransposeM {m: 11.0/16.0},
                        TransposeM {m: 15.0/16.0},
                        Silence {m: 0.25},
                        TransposeM {m: 7.0/8.0},
                        Silence {m: 0.25},
                        TransposeM {m: 7.0/8.0},
                        TransposeM {m: 15.0/16.0},
                    ],
                    Gain {m: 1.55}
                ]
            ],
            TransposeM {m: 0.5},
            Gain {m: 1.0}
        ]
    }

    fn bass_fit() -> Op {
        fit![
            bass() => sequence1(), 1
        ]
    }

    fn fit() -> Op {
        fit![
            compose![
                overtones(),
                sequence1(),
                TransposeM {m: 4.0/3.0},
                Gain {m: 0.35}
            ] => sequence1(), 10
        ]
    }

    fn fit2() -> Op {
        fit![
            compose![
                fit(),
                TransposeM {m: 5.0/4.0},
                Gain {m: 0.5}
            ] => sequence1(), 5
        ]
    }

    repeat![
        compose![
            repeat![
                overlay![
                    fit2(),
                    fit(),
                    bass_fit(),
                ], 1
            ],
            sequence![
                AsIs,
                AsIs,
                TransposeM {m: 9.0/8.0},
                AsIs,
                AsIs,
                TransposeM {m: 7.0/8.0},
            ]
        ], 4
    ]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(200.0, 0.75, 0.0, 1.5)
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
