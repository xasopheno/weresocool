use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Apply};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn overtones() -> Op {
        r![
            (6, 1, 11.0, 0.3, 0.0),
            (6, 1, 0.0, 0.3, 0.0),
            (5, 1, 0.0, 0.3, 0.0),
            (9, 2, 0.0, 0.3, 0.0),
            (9, 2, 5.0, 0.3, 0.0),
            (3, 1, 7.0, 0.14, 0.5),
            (3, 1, 0.0, 0.14, 0.5),
            (2, 1, 5.0, 0.1, -0.5),
            (2, 1, 0.0, 0.1, -0.5),
            (1, 1, 0.0, 1.0, 0.0),
            (1, 1, 3.0, 1.0, 0.0),
        ]
    }

    fn sequence1() -> Op {
        Op::Sequence {
            operations: vec![
                Op::TransposeM { m: 1.0/1.0 },
                Op::TransposeM { m: 9.0/8.0 },
                Op::TransposeM { m: 5.0/4.0 },
            ],
        }
    };

    fn sequence2() -> Op {
        Op::Sequence {
            operations: vec![
                Op::TransposeM { m: 1.0 / 1.0 },
                Op::TransposeM { m: 7.0 / 8.0 },
                Op::TransposeM { m: 4.0 / 5.0 },
            ],
        }
    }

    fn sequence3() -> Op {
        Op::Sequence {
            operations: vec![
                Op::TransposeM { m: 1.0 / 1.0 },
                Op::TransposeM { m: 4.0 / 3.0 },
                Op::TransposeM { m: 7.0 / 8.0 },
                Op::TransposeM { m: 6.0 / 5.0 },
            ],
        }
    }

    fn fit() -> Op {
        Op::Fit {
            n: 60,
            with_length_of: Box::new(sequences()),
            main: Box::new(
                Op::Compose { operations: vec![
                    overtones(),
                    sequence2(),
                    Op::TransposeM {m: 6.0 },
                    Op::Gain {m: 0.1}
                ]}
            )
        }
    }

    fn sequences() -> Op {
        Op::Compose {
            operations: vec![
                overtones(),
                sequence1(),
                sequence2(),
                sequence3(),
            ],
        }
    }

    fn overlay() -> Op {
        Op::Overlay { operations: vec![
            fit(),
            sequences()
        ]}
    }



    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(120.0, 1.0, 0.0, 0.8)];
    let mut events = overlay().apply(e);

    events.render(&mut oscillator)
}
