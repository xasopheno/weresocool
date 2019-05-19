mod bass;
mod cello;
mod flute_clarinet_basson;
mod horns;
mod violas;
mod violins;

use compositions::brahms::{
    bass::bass::bass1, cello::cello::cello1, flute_clarinet_basson::winds::winds1,
    horns::horns::horns1, violas::violas::violas1, violins::violins::violins1,
};
use event::{Event, Render};
use instrument::{Oscillator, StereoWaveform};
use operations::{Apply, Op};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn brahms() -> Op {
        Op::Sequence {
            operations: vec![Op::Overlay {
                operations: vec![violins1(), violas1(), winds1(), horns1(), cello1(), bass1()],
            }],
        }
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(164.8, 1.0, 0.0, 0.57)];
    let mut events = brahms().apply(e);

    events.render(&mut oscillator)
}
