use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let rs = r![
        (1, 2, 3.0, 0.5, 0.0),
        (1, 2, 0.0, 0.5, 0.0),
        (3, 4, 0.0, 1.0, 0.0),
        (3, 4, 3.0, 1.0, 0.0),
        (1, 1, 0.0, 1.0, 0.0),
        (1, 1, 3.0, 1.0, 0.0),
        (3, 2, 0.0, 1.0, -0.5),
        (3, 2, 6.0, 1.0, 0.5),
        (3, 2, 3.0, 1.0, 0.5),
        (5, 2, 0.0, 1.0, -0.2),
        (3, 2, 0.0, 1.0, 0.2),
        (4, 1, 12.0, 0.55, 0.2),
        (4, 1, 0.0, 0.55, -0.2),
        (6, 1, 12.0, 0.1, 0.3),
        (6, 1, 0.0, 0.1, -0.3),
        (10, 1, 12.0, 0.1, 0.4),
        (10, 1, 0.0, 0.1, -0.4),
    ];

    let e = vec![Event::new(120.0, rs.clone(), 0.75, 1.0)];
    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose {m: 6.0/5.0, a: 0.0},
            Op::Transpose {m: 3.0/2.0, a: 0.0},
            Op::Transpose {m: 5.0/4.0, a: 0.0},
        ],
    };

    let sequence2 = Op::Sequence {
        operations: vec![
            Op::Transpose {m: 25.0/24.0, a: 0.0},
            Op::Transpose {m: 7.0/6.0, a: 0.0},
            Op::Transpose {m: 25.0/24.0, a: 0.0},
            Op::Transpose {m: 4.0/3.0, a: 0.0},
            Op::Transpose {m: 7.0/6.0, a: 0.0},
            Op::Transpose {m: 2.0/1.0, a: 0.0},
            Op::Transpose {m: 4.0/3.0, a: 0.0},
        ],
    };

    let sequence3 = Op::Compose {
        operations: vec![
            sequence1.clone(),
            sequence2.clone(),
        ]
    };

    let sequence4 = Op::Sequence {
        operations: vec![
            sequence1.clone(),
            Op::Fit {
                with_length_of: Box::new(sequence1.clone()),
                main: Box::new(sequence3.clone())
            }
        ]
    };

    let sequence4 = Op::Sequence {
        operations: vec![
            sequence4.clone(),
            sequence4.clone(),
            sequence4.clone(),
            sequence4.clone(),
            sequence4.clone(),
            sequence4.clone(),
        ]
    };

    let mut oscillator = Oscillator::init(rs.clone(), &get_default_app_settings());
    let mut events = sequence4.apply(e);

    events.render(&mut oscillator)


}

