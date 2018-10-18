use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply};
use socool_parser::ast::{Op, Op::*};
use settings::get_default_app_settings;

fn composition() -> Op {
    fn _overtones() -> Op {
        r![
            (3, 2, 0.0, 1.0, -0.5),
            (3, 2, 3.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, -0.5),
            (1, 1, 3.0, 1.0, 0.5),
        ]
    }

    fn sequence1() -> Op {
        sequence![AsIs, TransposeM { m: 3.0 / 2.0 }, AsIs,]
    }

    compose![sequence1()]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(200.0, 0.75, 0.0, 1.8)
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
