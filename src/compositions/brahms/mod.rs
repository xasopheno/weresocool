mod violins;
mod flute_clarinet_basson;
mod horns;
mod cello;
mod bass;

use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Apply};
use settings::get_default_app_settings;
use compositions::brahms::{
    violins::violins::violins1,
    flute_clarinet_basson::winds::winds1,
    horns::horns::horns1,
    cello::cello::cello1,
    bass::bass::bass1,
};

pub fn generate_composition() -> StereoWaveform {
    fn brahms() -> Op {
        Op::Sequence {
            operations: vec![
                Op::Overlay { operations: vec![
                    violins1(),
                    winds1(),
                    horns1(),
                    cello1(),
                    bass1(),
                ]}
            ],
        }
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(164.8, 1.0, 0.0, 0.57)];
    let mut events = brahms().apply(e);

    events.render(&mut oscillator)
}
