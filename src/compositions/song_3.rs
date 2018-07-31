use event::{Event, Mutate, Phrase, Render};
use oscillator::{Oscillator, StereoWaveform};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = r![
        (5, 4, 0.0, 0.0, 0.0)
//        (1, 1, 0.0, 0.0, -0.0)
    ];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 230.0;
    let e = Event::new(freq, r.clone(), 2.0, 0.8);
    let phrase1 = Phrase {
        events: vec![
            e.clone().mut_ratios(
                r![
                    (5, 4, 0.0, 0.5, 1.0)
//                    (7, 8, 0.0, 0.5, -1.0)
                ]),
            e.clone().mut_ratios(
                r![
                    (4, 3, 0.0, 0.5, 0.5)
//                    (6, 9, 0.0, 0.5, -0.5)
                ]),
            e.clone().mut_ratios(
                r![
                    (3, 2, 0.0, 0.5, 0.25)
//                    (7, 8, 0.0, 0.5, 0.1)
                ]),
            e.clone().mut_ratios(
                r![
                    (4, 3, 0.0, 0.5, 0.0)
//                    (1, 1, 0.0, 0.5, 0.5)
                ]),
            e.clone().mut_ratios(
                r![
                    (5, 4, 0.0, 0.5, -0.5)
//                    (7, 8, 0.0, 0.5, 0.5)
                ]),
            e.clone().mut_ratios(
                r![
                    (4, 3, 0.0, 0.5, -1.0)
//                    (6, 9, 0.0, 0.5, 0.75)
                ]),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![phrase1.clone(), phrase1.clone(), end].render(&mut oscillator)
}
