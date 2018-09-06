mod area1;
mod area2;
mod area3;

use compositions::song_15::{
    area1::material::{fit, fit_again, repeat, sequence1, sequence2}, area2::material2::sequence4,
    area3::material3::sequence5,
};

use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Apply, Op};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn intro_overtones() -> Op {
        r![
            (23, 1, 4.0, 0.01, -1.0),
            (23, 1, 4.0, 0.01, 1.0),
            (20, 1, 4.0, 0.015, 1.0),
            (20, 1, 3.0, 0.015, 1.0),
            (17, 1, 0.5, 0.04, -0.7),
            (17, 1, 0.5, 0.04, -0.7),
            (15, 1, 0.1, 0.08, 0.7),
            (15, 1, 0.1, 0.08, 0.7),
            (12, 1, 0.1, 0.1, 0.7),
            (12, 1, 0.1, 0.1, -0.7),
            (10, 1, 0.2, 0.3, -0.5),
            (10, 1, 0.2, 0.3, -0.5),
            (9, 1, 0.3, 0.3, 0.5),
            (9, 1, 0.3, 0.3, -0.5),
            (6, 1, 7.0, 1.0, 0.5),
            (6, 1, 0.0, 1.0, 0.5),
            (5, 1, 3.0, 1.0, -1.0),
            (5, 1, 0.0, 1.0, 1.0),
            (3, 1, 4.0, 1.0, -0.5),
            (3, 1, 0.0, 1.0, 0.5),
            (3, 2, 0.0, 0.3, 0.0),
            (3, 2, 5.0, 0.3, 0.0),
            (1, 1, 0.0, 1.0, 0.0),
            (1, 1, 5.0, 1.0, 0.0),
            (1, 2, 5.0, 1.0, 0.0),
            (1, 2, 0.0, 1.0, 0.0),
        ]
    }

    fn intro() -> Op {
        Op::Compose {
            operations: vec![
                intro_overtones(),
                sequence2(),
                Op::Length { m: 5.5 },
                Op::Gain { m: 0.25 },
                Op::TransposeA { a: 5.0 },
            ],
        }
    }

    fn fit_test() -> Op {
        Op::Fit {
            with_length_of: Box::new(sequence4()),
            n: 1,
            main: Box::new(Op::Compose {
                operations: vec![
                    Op::Sequence {
                        operations: vec![Op::Silence { m: 10.0 }, fit(), Op::Silence { m: 12.0 }],
                    },
                    Op::TransposeM { m: 1.8 },
                    Op::Gain { m: 0.1 },
                    Op::Reverse {},
                ],
            }),
        }
    }

    fn form() -> Op {
        Op::Sequence {
            operations: vec![
                Op::Compose {
                    operations: vec![
                        sequence5(),
                        Op::Gain { m: 0.4 },
                        Op::Length { m: 0.6666 },
                        //                    Op::TransposeM { m: 25.0/24.0 }
                    ],
                },
                Op::Overlay {
                    operations: vec![
                        sequence4(),
                        Op::Compose {
                            operations: vec![fit_test(), Op::Gain { m: 2.6 }],
                        },
                    ],
                },
                repeat(),
                Op::AsIs,
                Op::AsIs,
            ],
        }
    }

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(100.0, 1.0, 0.0, 1.25)];
    let mut events = form().apply(e);

    events.render(&mut oscillator)
}
