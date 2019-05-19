use event::{Event, Render};
use instrument::{Oscillator, StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;
use compositions::{
    song_18::operations as song_18,
    song_19::{
        operations as song_19,
    }
};

fn composition() -> Op {
    fn overtones() -> Op {
        r![
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, -0.5),
            (1, 1, 3.0, 1.0, 0.5),
        ]
    }

    fn form() -> Op {
        repeat![
            sequence![
                compose![
                    song_18(),
                    Length {m: 1.53}
                ],
                compose![
                    song_19(),
                    Length {m: 2.0},
                    Gain {m: 0.90}
                ]
            ], 3
        ]
    }

    compose![form()]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(190.0, 0.75, 0.0, 1.0)
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
