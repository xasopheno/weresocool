use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{
    Apply, 
    Op,
    Op::*,
};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    fn ratios() -> Op {
        r![
            (3, 2, 1.0, 0.1, -1.0),
            (3, 2, 0.0, 0.1, 1.0),
            (1, 1, 0.0, 1.0, 0.0),
        ]
    }
    
    fn sequence1() -> Op {
        compose![
            ratios(),
            sequence![
                TransposeM { m: 1.0 / 1.0 },
                TransposeM { m: 9.0 / 8.0 },
                TransposeM { m: 5.0 / 4.0 },
                TransposeM { m: 3.0 / 2.0 },
                Silence { m: 1.0 },
            ],
        ]
    };

    fn fractal(depth: usize) -> Op {
        let mut count = 1;
        let mut result = sequence1();
        while count < depth {
            let new_result = compose![
                Op::Fit {
                    n: count * 3,
                    with_length_of: Box::new( sequence1() ),
                    main: Box::new(
                        compose![
                            sequence1(),
                            TransposeM { m: count as f32 * 3.0 / 2.0 },
                            Gain { m: 1.0 / (3.0 * count as f32) },
                            Reverse {},
                        ],
                    ),
                }
            ];
            result = overlay![result, new_result];
            count += 1
        }
        result
    }

    fn main() -> Op {
        sequence![
            fractal(20),
            Silence { m: 0.1 },
            fractal(10),
            Silence { m: 0.1 },
            fractal(13),
            Silence { m: 0.1 },
            fractal(16),
            Silence { m: 0.1 },
            fractal(10),
        ]
    }

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(120.0, 0.75, 0.0, 4.0)];

    let mut events = main().apply(e);

    events.render(&mut oscillator)
}
