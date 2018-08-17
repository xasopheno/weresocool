use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let sequence1 = Op::Sequence {
        operations: vec![Op::AsIs],
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(100.0, 1.0, 1.0, 1.0)];
    let mut events = sequence1.apply(e);

    events.render(&mut oscillator)
}
