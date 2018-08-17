use event::{Event, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use operations::{Op, Operate};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let sequence1 = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose {m: 2.0, a: 0.0},
            Op::Length {m: 0.5},
            Op::Transpose {m: 1.5, a: 0.0},
            Op::Gain {m: 0.5},
            Op::Transpose {m: 5.0/4.0, a: 0.0},
            Op::AsIs,
        ],
    };

    let sequence2 = Op::Compose { operations: vec![
            sequence1.clone(),
            Op::Length { m: 0.5},
            Op::Sequence { operations: vec![
                Op::AsIs,
                Op::Transpose {m: 9.0/8.0, a: 0.0},
                Op::Transpose {m: 6.0/5.0, a: 0.0},
            ]},
            Op::Sequence { operations: vec![
                Op::AsIs,
                Op::Transpose {m: 9.0/8.0, a: 0.0},
                Op::Transpose {m: 6.0/5.0, a: 0.0},
            ]}
        ]
    };

    let mut oscillator = Oscillator::init(&get_default_app_settings());
    let e = vec![Event::init(300.0, 1.0, 0.0, 1.0)];
    let mut events = sequence2.apply(e);

    events.render(&mut oscillator)
}
