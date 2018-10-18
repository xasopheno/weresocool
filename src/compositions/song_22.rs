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
        sequence![
            AsIs,
            TransposeM { m: 3.0 / 2.0 },
            AsIs,
        ]
    }

    fn sequence2() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 7.0 / 8.0 },
            TransposeM { m: 4.0 / 5.0 },
        ]
    }

    fn sequence3() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 4.0 / 3.0 },
            TransposeM { m: 3.0 / 2.0 },
            TransposeM { m: 15.0 / 8.0 },
        ]
    }

    fn melody() -> Op {
            compose![
            sequence1(),
            sequence2(),
            sequence3(),
        ]
    }

    fn fit() -> Op {
        fit![
            sequence2() => melody(), 7
        ]
    }

    overlay![
        melody(),
        fit()
    ]
}

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event() -> Event {
    Event::init(400.0, 0.75, 0.0, 0.5)
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
