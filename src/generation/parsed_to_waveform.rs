extern crate num_rational;
extern crate socool_parser;
use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use num_rational::Rational;
use operations::Apply;
use settings::get_default_app_settings;
use socool_parser::{ast::Op, parser::Init};

fn oscillator() -> Oscillator {
    Oscillator::init(&get_default_app_settings())
}

pub fn r_to_f32(r: Rational) -> f32 {
//    let x = 100000000000000000000000000000000000000;
//    if r.numer() > &x || r.denom() > &x {
//        let mut fract = r.fract();
//        for _ in 0..32 {
//            if fract.is_integer() {
//                break;
//            }
//            fract = fract * 10
//        }
//
//        return r.to_integer() as f32 + fract.to_integer() as f32;
//    }

    simple_r_to_f32(r)
}

fn simple_r_to_f32(r: Rational) -> f32 {
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

pub fn generate_events(init: Init, main: Op) -> Vec<Event> {
    main.apply(vec![event_from_init(init)])
}

pub fn generate_composition(init: Init, main: Op) -> StereoWaveform {
    generate_events(init, main).render(&mut oscillator())
}
