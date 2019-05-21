use event::{Event, Render};
use instrument::{Oscillator, StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;

fn composition() -> Op {
    fn overtones() -> Op {
        r![
            (9, 2, 0.0, 0.2, -1.0),
            (5, 2, 5.0, 0.2, -1.0),
            (5, 2, 5.0, 0.2, 1.0),
            (5, 2, 0.0, 0.2, -0.5),
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 3.0, 1.0, 0.0),
            (1, 1, 0.0, 1.0, 0.0),
            (1, 2, 0.0, 1.0, 0.0),
            (1, 2, 3.0, 1.0, 0.0),
        ]
    }

    fn sequence1() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 3.0 / 2.0 },
            Silence {m: 1.0},
            TransposeM { m: 9.0 / 8.0 },
            Silence {m: 1.5},
        ]
    }

    fn sequence2() -> Op {
        sequence![
            Silence {m: 1.0},
            Silence {m: 1.0},
            TransposeM { m: 7.0 / 8.0 },
            Silence {m: 1.0},
            TransposeM { m: 4.0 / 5.0 },
            TransposeM { m: 3.0 / 4.0 },
        ]
    }


    fn sequence3() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 3.0 / 2.0 },
            TransposeM { m: 7.0 / 8.0 },
            TransposeM { m: 6.0 / 5.0 },
        ]
    }


    fn thing() -> Op {
        compose![
            overtones(),
            sequence1(),
            sequence2(),
            sequence3(),
            Length {m: 1.0/2.0},
            TransposeM {m: 2.0/1.0},
            Gain {m: 0.5}
        ]
    }

    fn thing2() -> Op {
        compose![
            r![
                (3, 4, 0.0, 1.0, -0.5),
                (3, 4, 3.0, 1.0, 0.5),
                (1, 2, 3.0, 1.0, 0.0),
                (1, 2, 0.0, 1.0, 0.0),
            ],
            sequence![
                AsIs,
                AsIs,
                TransposeM { m: 6.0 / 5.0 },
                TransposeM { m: 4.0 / 3.0 },
                TransposeM { m: 11.0 / 8.0 },
                TransposeM { m: 11.0 / 8.0 },
                TransposeM { m: 3.0 / 4.0 },
            ],
        ]
    }

    fn fit_thing2() -> Op {
        fit![
            thing2() => thing(), 2
        ]
    }

    repeat![
        overlay![
            fit![
                compose![
                    thing(),
                    TransposeM {m: 9.0/4.0},
                    Gain {m: 0.3},
                    Reverse,
                ] => thing(), 3
            ],
            thing(),
            fit_thing2()
        ], 2
    ]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(200.0, 0.75, 0.0, 0.20)
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
