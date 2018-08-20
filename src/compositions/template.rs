use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn macro_test() -> Op {
        r![
            (1, 1, 0.0, 1.0, 0.0),
            (1, 1, 3.0, 1.0, 0.0),
            (3, 1, 1.0, 0.14, 0.5),
            (3, 1, 0.0, 0.14, 0.5),
            (2, 1, 0.0, 0.1, -0.5),
            (2, 1, 5.0, 0.1, -0.5),
        ]
    }

    fn sequence1() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
            ],
        }
    };

    let main = Op::Compose {
        operations: vec![macro_test(), sequence1()],
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(120.0, 1.0, 0.0, 0.8)];
    let mut events = main.apply(e);

    events.render(&mut oscillator)
}
