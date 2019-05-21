use event::{Event, Render};
use instrument::{Oscillator, StereoWaveform};
use operations::{Apply, Op};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (1, 1, 0.0, 1.0, 0.0),
            (1, 1, 3.0, 1.0, 0.0),
            (1, 1, 5.0, 1.0, 0.0),
            (3, 1, 1.0, 0.14, 0.5),
            (3, 1, 0.0, 0.14, 0.5),
            (2, 1, 0.0, 0.1, -0.5),
            (2, 1, 5.0, 0.1, -0.5),
        ]
    }

    fn sequence1() -> Op {
        Op::Sequence {
            operations: vec![Op::AsIs],
        }
    };

    fn with_overtones() -> Op {
        Op::Compose {
            operations: vec![overtones(), sequence1()],
        }
    }

    fn order_fn(order: usize, _length: usize) -> f32 {
        order as f32
    }

    let main = Op::ComposeWithOrder {
        operations: vec![
            Op::Repeat {
                n: 10,
                operations: vec![with_overtones()],
            },
            Op::TransposeA { a: 100.0 },
        ],
        order_fn,
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(120.0, 1.0, 0.0, 0.5)];
    let mut events = main.apply(e);

    events.render(&mut oscillator)
}
