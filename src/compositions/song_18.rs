use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply, Op, Op::*};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (3, 1, 1.0, 0.14, -0.5),
            (3, 1, 0.0, 0.14, 0.5),
            (2, 1, 5.0, 0.1, 0.5),
            (2, 1, 0.0, 0.1, -0.5),
            (1, 1, 0.0, 1.0, 0.0),
            (1, 1, 3.0, 1.0, 0.0),
            (1, 2, 5.0, 1.0, 0.0),
            (1, 2, 0.0, 1.0, 0.0),
        ]
    }

    fn sequence1() -> Op {
        sequence![AsIs, AsIs, TransposeM { m: 3.0 / 2.0 },]
    };

    fn sequence2() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 4.0 / 5.0 },
            TransposeM { m: 7.0 / 8.0 },
        ]
    };

    fn sequence3() -> Op {
        sequence![
            AsIs,
            TransposeM { m: 9.0 / 8.0 },
            TransposeM { m: 4.0 / 3.0 },
            TransposeM { m: 5.0 / 4.0 },
            TransposeM { m: 5.0 / 4.0 },
            TransposeM { m: 5.0 / 3.0 },
            TransposeM { m: 3.0 / 2.0 },
            TransposeM { m: 3.0 / 2.0 },
            TransposeM { m: 2.0 / 1.0 },
            TransposeM { m: 2.0 / 1.0 },
            TransposeM { m: 15.0 / 8.0 },
            TransposeM { m: 15.0 / 8.0 },
        ]
    };

    fn fit() -> Op {
        fit![
            compose![
                main(),
                TransposeM {m: 3.0/ 2.0}
            ]
            => main(), 3
        ]
    }

    fn main() -> Op {
        compose![sequence3(), sequence2(), compose![sequence3(), Reverse {},],]
    };

    fn overlay() -> Op {
        overlay![fit(), compose![overtones(), main(),]]
    }

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(220.0, 1.0, 0.0, 0.25)];
    let mut events = overlay().apply(e);

    events.render(&mut oscillator)
}
