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

    let e = vec![Event::new(130.0, rs.clone(), 0.75, 0.3)];
    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose {
                m: 6.0 / 5.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 3.0 / 2.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 5.0 / 4.0,
                a: 0.0,
            },
        ],
    };

    let sequence2 = Op::Sequence {
        operations: vec![
            Op::Transpose {
                m: 25.0 / 24.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 7.0 / 6.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 25.0 / 24.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 7.0 / 6.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 2.0 / 1.0,
                a: 0.0,
            },
            Op::Transpose {
                m: 4.0 / 3.0,
                a: 0.0,
            },
        ],
    };

    let sequence3 = Op::Compose {
        operations: vec![sequence1.clone(), sequence2.clone()],
    };

    let sequence4 = Op::Sequence {
        operations: vec![
            Op::Fit {
                with_length_of: Box::new(sequence1.clone()),
                main: Box::new(sequence3.clone()),
            },
            sequence1.clone(),
            sequence1.clone(),
            sequence1.clone(),
        ],
    };

    let op_ratios = Op::Ratios {
        ratios: r![
            (1, 1, 8.0, 0.5, 0.0),
            (15, 8, 0.0, 0.5, 0.0),
            (2, 1, 0.0, 1.0, 0.0),
            (1, 1, 8.0, 1.0, 0.0),
            (5, 2, 0.0, 1.0, 0.0),
            (1, 1, 0.0, 1.0, 0.0),
            (7, 6, 0.0, 1.0, -0.5),
            (3, 2, 6.0, 1.0, 0.5),
            (11, 2, 5.0, 1.0, 0.5),
            (5, 2, 0.0, 1.0, -0.2),
            (16, 5, 0.0, 1.0, 0.2),
            (16, 2, 12.0, 0.55, 0.2),
            (4, 1, 0.0, 0.55, -0.2),
            (6, 1, 12.0, 0.1, 0.3),
            (6, 1, 0.0, 0.1, -0.3),
            (10, 1, 12.0, 0.1, 0.4),
            (10, 1, 0.0, 0.1, -0.4),
        ],
    };

    let sequence5 = Op::Sequence {
        operations: vec![
            op_ratios.clone(),
            sequence4.clone(),
            Op::Compose {
                operations: vec![op_ratios.clone(), Op::Length { m: 1.5 }],
            },
            sequence4.clone(),
            op_ratios.clone(),
        ],
    };

    let mut oscillator = Oscillator::init(rs.clone(), &get_default_app_settings());
    let mut events = sequence5.apply(e);

    events.render(&mut oscillator)
}
