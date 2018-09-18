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

    fn sequence1() -> Op {
        compose![
            sequence![
                TransposeM { m: 5.0 / 4.0 },
                TransposeM { m: 9.0 / 8.0 },
            ],
            Length {m: 0.5}
        ]
    }

    fn sequence2() -> Op {
        compose![
            sequence![
                TransposeM { m: 3.0 / 2.0 },
                TransposeM { m: 5.0 / 4.0 },
                TransposeM { m: 3.0 / 2.0 },
            ],
            Length {m: 0.3333}
        ]
    }

    fn sequence3() -> Op {
        compose![
            sequence![
                TransposeM { m: 2.0 / 1.0 },
                Silence {m: 1.0},
                TransposeM { m: 5.0 / 3.0 },
                TransposeM { m: 3.0 / 2.0 },
                Silence {m: 1.0},
                TransposeM { m: 9.0 / 4.0 },
            ],
            Length {m: 0.3333 / 2.0}
        ]
    }

    fn sequence4() -> Op {
        compose![
            sequence![
                TransposeM { m: 5.0 / 2.0 },
                Silence {m: 1.0},
                TransposeM { m: 1.0 / 1.0 },
                TransposeM { m: 1.0 / 1.0 },
                Silence {m: 1.0},
                TransposeM { m: 8.0 / 3.0 },
            ],
            Length {m: 0.3333 / 2.0}
        ]
    }

    fn sequence5() -> Op {
        compose![
            sequence![
                TransposeM { m: 2.0 / 3.0 },
                TransposeM { m: 3.0 / 4.0 },
                Silence {m: 1.0 },
                TransposeM { m: 3.0 / 4.0 },
                TransposeM { m: 3.0 / 4.0 },
            ],
            TransposeM {m: 0.5},
            Length {m: 1.0 / 5.0}
        ]
    }

    fn sequence6() -> Op {
        compose![
            r![
                (15, 1, 2.0, 0.1, 1.0),
                (14, 1, 3.0, 0.1, -1.0),
                (13, 1, 0.0, 0.1, 1.0),
                (12, 1, 4.0, 0.1, -1.0),
                (11, 1, 0.0, 0.3, 0.0),
                (10, 1, 5.0, 1.0, 0.5),
                (9, 1, 0.0, 1.0, -0.5),
            ],
            Gain {m: 0.08}
        ]
    }

    fn sequence7() -> Op {
        compose![
            sequence![
                Silence {m: 0.5},
                r![
                    (6, 1, 3.0, 0.5, -1.0),
                    (6, 1, 0.0, 0.5, 1.0),
                    (4, 1, 0.0, 1.0, 0.0),
                    (8, 3, 0.0, 1.0, 0.0),
                    (4, 3, 0.0, 1.0, 0.0),
                    (9, 4, 0.0, 1.0, 0.0),
                ],
                r![
                    (5, 1, 3.0, 0.5, -1.0),
                    (5, 1, 0.0, 0.5, 1.0),
                    (4, 1, 0.0, 1.0, 0.0),
                    (3, 1, 0.0, 1.0, 0.0),
                    (5, 2, 0.0, 1.0, 0.0),
                    (2, 1, 0.0, 1.0, 0.0),
                ],
                r![
                    (4, 1, 3.0, 0.5, -1.0),
                    (4, 1, 0.0, 0.5, 1.0),
                    (3, 1, 0.0, 1.0, 0.0),
                    (5, 2, 0.0, 1.0, 0.0),
                    (2, 1, 0.0, 1.0, 0.0),
                    (5, 3, 0.0, 1.0, 0.0),
                ],
                r![
                    (3, 1, 3.0, 0.5, -1.0),
                    (3, 1, 0.0, 0.5, 1.0),
                    (5, 2, 0.0, 1.0, 0.0),
                    (2, 1, 0.0, 1.0, 0.0),
                    (5, 3, 0.0, 1.0, 0.0),
                    (3, 2, 0.0, 1.0, 0.0),
                ],
                r![
                    (2, 1, 3.0, 0.5, -1.0),
                    (2, 1, 0.0, 0.5, 1.0),
                    (5, 2, 0.0, 1.0, 0.0),
                    (15, 8, 0.0, 1.0, 0.0),
                    (3, 2, 0.0, 1.0, 0.0),
                    (5, 4, 0.0, 1.0, 0.0),
                ],
                r![
                    (1, 1, 3.0, 0.5, -1.0),
                    (1, 1, 0.0, 0.5, 1.0),
                    (5, 2, 0.0, 1.0, 0.0),
                    (2, 1, 0.0, 1.0, 0.0),
                    (3, 2, 0.0, 1.0, 0.0),
                    (5, 4, 0.0, 1.0, 0.0),
                ],
                Silence {m: 0.5},
            ],
            Gain {m: 0.25},
            Length {m: 1.0 / 7.0}
        ]
    }

    fn tag() -> Op {
        sequence![
            compose![
                    TransposeM {m: 3.0/8.0},
                    Length {m: 1.0/8.0},
                    Gain {m: 2.3},
                ],
            Silence {m: 0.1},
            overlay![
                    sequence7(),
                    compose![
                        sequence7(),
                        TransposeM {m: 2.0/1.0},
                        PanA {a: 0.5},
                    ],
                    compose![
                        sequence7(),
                        TransposeM {m: 5.0/2.0},
                        PanA {a: -0.5},
                    ],
                    compose![
                        sequence7(),
                        TransposeM {m: 3.0/2.0},
                    ],
                ],
            compose![
                    TransposeM {m: 1.0/2.0},
                    Length {m: 1.0/8.0},
                    Gain {m: 2.0},
                ],
            Silence {m: 0.1}
        ]
    }

    fn tag_fit() -> Op {
        fit![
            tag() => sequence![sequence1()], 1
        ]
    }

    sequence![
        tag_fit(),
        repeat![
            sequence![
                repeat![
                    compose![
                        r![
                            (10, 1, 3.0, 0.002, 1.0),
                            (10, 1, 0.0, 0.002, -1.0),
                            (1, 1, 1.0, 1.0, -0.5),
                            (1, 1, 0.0, 1.0, 0.5),
                        ],
                        overlay![
                            sequence1(),
                            sequence2(),
                            sequence3(),
                            sequence4(),
                            sequence5(),
                            sequence6(),
                        ]
                    ], 7
                ],
                tag_fit()
            ], 8
        ]
    ]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(200.0, 1.0, 0.0, 2.0)
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
