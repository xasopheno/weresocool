use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let rs = r![
        (1, 1, 0.0, 1.0, 0.0),
    ];

    let e = vec![Event::new(120.0, rs.clone(), 0.5, 1.0)];
    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose {m: 5.0/4.0, a: 0.0},
            Op::Transpose {m: 3.0/2.0, a: 0.0},
            Op::Transpose {m: 9.0/8.0, a: 0.0},
        ],
    };

    let sequence2 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::AsIs,
        ],
    };

    let sequence3 = Op::Compose {
        operations: vec![
            sequence1.clone(),
            sequence2.clone()
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

    let mut oscillator = Oscillator::init(rs.clone(), &get_default_app_settings());
    let mut events = sequence4.apply(e);

    events.render(&mut oscillator)
}git 

