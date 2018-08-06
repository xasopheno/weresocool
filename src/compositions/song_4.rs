use event::{Event, Mutate, Phrase, Render};
use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = r![
        (5, 4, 0.0, 0.6, -1.0),
        (1, 2, -1.0, 0.6, 1.0),
        (0, 2, -1.0, 0.0, 1.0),
        (0, 2, -1.0, 0.0, 1.0),
    ];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 230.0;
    let e = Event::new(freq, r.clone(), 1.4, 0.8);
    let phrase1 = Phrase {
        events: vec![
            e.clone().mut_ratios(r![
                (3, 2, 0.0, 0.6, 1.0),
                (5, 4, 0.0, 0.6, -1.0),
                (0, 2, -1.0, 0.0, 1.0),
                (0, 2, -1.0, 0.0, 1.0),
            ]),
            e.clone().mut_ratios(r![
                (2, 3, 0.0, 0.6, -1.0),
                (5, 4, 11.0, 0.6, 1.0),
                (2, 1, 11.0, 0.6, 1.0),
                (0, 2, -1.0, 0.0, 1.0),
            ]),
            e.clone().mut_ratios(r![
                (3, 2, 0.0, 0.6, 1.0),
                (5, 4, 5.0, 0.6, -1.0),
                (0, 2, -1.0, 0.0, 1.0),
                (0, 2, -1.0, 0.0, 1.0),
            ]),
            e.clone().mut_ratios(r![
                (5, 4, -10.0, 0.6, -1.0),
                (1, 2, -1.0, 0.6, 1.0),
                (2, 1, -7.0, 0.6, 0.0),
                (2, 1, -1.0, 0.6, 1.0),
            ]),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![
        phrase1.clone(),
        phrase1.clone().transpose(9.0 / 8.0, 0.0),
        phrase1.clone(),
        phrase1.clone().transpose(15.0 / 16.0, 0.0),
        phrase1.clone(),
        phrase1.clone().transpose(9.20 / 8.0, 0.0),
        phrase1.clone(),
        phrase1.clone().mut_ratios(r![
            (11, 8, -10.0, 0.6, -1.0),
            (7, 4, 5.0, 0.6, 1.0),
            (2, 1, -7.0, 0.6, 0.0),
            (7, 2, -1.0, 0.3, 1.0),
        ]),
        phrase1.clone(),
        phrase1.clone().transpose(9.0 / 8.0, 0.0),
        phrase1.clone(),
        phrase1.clone().transpose(14.8 / 16.0, 0.0),
        phrase1.clone(),
        phrase1.clone().transpose(9.25 / 8.0, 0.0),
        phrase1.clone(),
        phrase1.clone().mut_ratios(r![
            (11, 8, -10.0, 0.6, 1.0),
            (7, 4, 5.0, 0.6, -1.0),
            (2, 1, -7.0, 0.6, 0.0),
            (7, 2, -1.0, 0.3, -1.0),
        ]),
        end,
    ].render(&mut oscillator)
}
