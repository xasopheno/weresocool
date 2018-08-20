use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn ratios() -> Op {
        r![
            (3, 2, 3.0, 0.1, -1.0),
            (3, 2, 0.0, 0.1, 1.0),
            (1, 1, 1.0, 1.0, 0.5),
            (1, 1, 0.0, 1.0, 0.5),
        ]
    }

    fn sequence1() -> Op {
        Op:: Compose { operations: vec![
            ratios(),
            Op::Sequence {
                operations: vec![
                    Op::AsIs,
                    Op::TransposeM { m: 9.0/8.0 },
                    Op::TransposeM { m: 5.0/4.0 },
                    Op::TransposeM { m: 3.0/2.0 },
                ],
            }
        ]}
    };

    fn fractal(depth: usize) -> Op {
        let mut count = 1;
        let mut result = sequence1();
        while count < depth {
            let new_result =
                Op::Compose { operations: vec![
                    Op::Fit {
                        n: count * 3,
                        with_length_of: Box::new(sequence1().clone()),
                        main: Box::new(
                        Op::Compose { operations: vec![
                                sequence1(),
                                Op::TransposeM { m: count as f32 * 1.5 },
                                Op::Gain { m: 1.0/(3.0 * count as f32) },
                                Op::Reverse {},
                            ]
                        },
                    ),
                },
                ],

            };
            result = Op::Overlay {operations: vec![
                result,
                new_result
            ]};
            count += 1
        }
        result
    }

    let main = Op::Sequence {
        operations: vec![
            fractal(10),
            Op::Silence { m: 0.75 },
            fractal(17),
            Op::Silence { m: 0.75 },
            fractal(20),
        ],
    };

    println!("{:?}", main);

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(120.0, 1.0, 0.0, 4.0)];
    let mut events = main.apply(e);

    events.render(&mut oscillator)
}
