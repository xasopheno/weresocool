extern crate socool_parser;
use socool_parser::{ast::Op, parser::Init};

use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::Apply;
use settings::get_default_app_settings;

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

fn event_from_init(init: Init) -> Event {
    Event::init(init.f, init.g, init.p, init.l)
}

fn generate_events(init: Init, main: Op) -> Vec<Event> {
    main.apply(vec![event_from_init(init)])
}

pub fn generate_composition(init: Init, main: Op) -> StereoWaveform {
    generate_events(init, main).render(&mut oscillator())
}
