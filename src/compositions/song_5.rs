use event::{Event, Render};
use instrument::{Oscillator, StereoWaveform};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = r![
        (1, 1, 1.0, 0.6, 0.5),
        (1, 1, 3.0, 0.6, 0.5),
        (1, 1, 4.0, 0.6, 0.5),
        (1, 1, -5.0, 0.6, 0.5),
        (0, 1, -5.0, 0.6, 0.5),
        (0, 1, -5.0, 0.6, 0.5)
    ];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 200.0;
    let e = Event::new(freq, r.clone(), 0.25, 0.8);
    let phrase1 = Phrase {
        events: vec![
            e.clone().transpose(8.0 / 5.0, 0.0),
            e.clone().transpose(4.0 / 3.0, 0.0),
            e.clone(),
            e.clone().transpose(7.0 / 8.0, 0.0),
            e.clone().transpose(3.0 / 4.0, 0.0),
            e.clone().transpose(5.0 / 3.0, 0.0),
            e.clone().transpose(1.0 / 2.0, 0.0),
            e.clone().transpose(2.0 / 3.0, 0.0),
            e.clone().transpose(5.0 / 4.0, 0.0),
            e.clone().transpose(9.0 / 8.0, 0.0),
            e.clone().mut_length(5.0, 0.0).transpose(5.0 / 4.0, 0.0),
            e.clone().mut_length(4.0, 0.0).transpose(0.0 / 1.0, 0.0),
        ],
    };

    let r2 = r![
        (3, 2, 1.0, 0.6, 0.5),
        (1, 1, 2.0, 0.6, 0.5),
        (1, 1, 3.0, 0.6, 0.5),
        (2, 1, -2.0, 0.6, 0.5),
        (0, 1, 0.0, 0.0, 0.0),
        (0, 1, 0.0, 0.0, 0.0)
    ];

    let r3 = r![
        (3, 2, 1.0, 0.6, 0.5),
        (1, 1, 2.0, 0.6, -0.5),
        (1, 1, 3.0, 0.6, -0.5),
        (2, 1, -2.0, 0.6, 0.5),
        (11, 8, 3.0, 0.6, -0.8),
        (7, 4, -2.0, 0.6, 0.5)
    ];

    let r4 = r![
        (1, 1, 0.0, 0.6, 0.2),
        (1, 1, 0.0, 0.6, -0.7),
        (2, 1, 0.0, 0.6, 0.5),
        (2, 1, 5.0, 0.6, -0.7),
        (3, 2, 0.0, 0.6, 0.5),
        (10, 4, 0.0, 0.6, -0.4),
    ];

    let r5 = r![
        (1, 1, 0.0, 0.6, -0.5),
        (1, 1, 0.0, 0.6, 0.7),
        (2, 1, 0.0, 0.6, -0.5),
        (2, 1, 5.0, 0.6, 0.7),
        (3, 2, 0.0, 0.6, -0.5),
        (10, 4, 0.0, 0.6, 0.4),
    ];

    let e2 = Event::new(freq, r3.clone(), 0.5, 0.8);
    let phrase2 = Phrase {
        events: vec![
            e2.clone().mut_length(0.25, 0.0).transpose(7.0 / 8.0, 0.0),
            e2.clone().mut_length(2.0, 0.0).transpose(5.0 / 4.0, 0.0),
            e2.clone().mut_length(2.0, 0.0).transpose(11.0 / 4.0, 0.0),
            e2.clone().mut_length(2.0, 0.0).transpose(8.0 / 5.0, 0.0),
            e2.clone()
                .mut_length(2.0, 0.0)
                .transpose(8.0 / 5.0, 0.0)
                .transpose(7.0 / 8.0, 0.0),
            e2.clone()
                .mut_length(2.0, 0.0)
                .transpose(8.0 / 5.0, 0.0)
                .transpose(11.0 / 8.0, 0.0),
            e2.clone()
                .mut_length(2.0, 0.0)
                .transpose(8.0 / 5.0, 0.0)
                .transpose(3.0 / 2.0, 0.0),
        ],
    };

    let e3 = Event::new(freq, r5.clone(), 0.5, 0.8);
    let phrase3 = Phrase {
        events: vec![
            e3.clone().mut_length(1.5, 0.0),
            e3.clone().transpose(15.0 / 16.0, 0.0).mut_length(1.5, 0.0),
            e3.clone().mut_length(2.0, 0.0).transpose(1.0 / 1.0, 0.0),
            e3.clone().transpose(9.0 / 8.0, 0.0),
            e3.clone().transpose(5.0 / 4.0, 0.0),
            e3.clone().mut_ratios(r4.clone()).transpose(4.0 / 3.0, 0.0),
            e3.clone().mut_ratios(r4.clone()).transpose(5.0 / 4.0, 0.0),
            e3.clone()
                .mut_length(15.0, 0.0)
                .mut_ratios(r.clone())
                .transpose(9.0 / 8.0, 0.0),
            e3.clone().mut_length(10.0, 0.0).mut_ratios(r2.clone()),
            e3.clone().mut_length(2.0, 0.0).mut_ratios(r2.clone()),
            e3.clone()
                .mut_length(25.0, 0.0)
                .mut_ratios(r3.clone())
                .transpose(9.0 / 8.0, 0.0),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![
        phrase1.clone(),
        phrase1.clone().mut_ratios(r2.clone()),
        phrase1.clone().mut_ratios(r3.clone()),
        phrase1.clone().mut_ratios(r4.clone()),
        phrase1.clone().mut_ratios(r5),
        phrase2.clone(),
        phrase3.clone(),
        //        phrase1.clone(),
        //        phrase1.clone(),
        end,
    ].render(&mut oscillator)
}
