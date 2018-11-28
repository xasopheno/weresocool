extern crate num_rational;
extern crate socool_parser;
use event::Event;
use instrument::oscillator::Oscillator;
use num_rational::Rational;
//use operations::Apply;
use settings::get_default_app_settings;
use socool_parser::parser::Init;

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

pub fn r_to_f32(r: Rational) -> f32 {
    *r.numer() as f32 / *r.denom() as f32
}

pub fn event_from_init(init: Init) -> Event {
    Event::init(
        r_to_f32(init.f),
        r_to_f32(init.g),
        r_to_f32(init.p),
        r_to_f32(init.l),
    )
}
