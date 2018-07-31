use event::{Event, Mutate, Phrase, Render};
use oscillator::{Oscillator, StereoWaveform};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = r![
        (5, 4, 0.0, 0.6, -1.0)
        (1, 2, -1.0, 0.6, 1.0)
    ];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 230.0;
    let e = Event::new(freq, r.clone(), 1.5, 0.8);
    let phrase1 = Phrase {
        events: vec![
            e.clone()
                .mut_ratios(
                    r![
                    (3, 2, 0.0, 0.6, 1.0)
                    (5, 4, 0.0, 0.6, -1.0)
                ]),
            e.clone()
                .mut_ratios(
                    r![
                    (2, 3, 0.0, 0.6, -1.0)
                    (5, 4, 11.0, 0.6, 1.0)
                ]),
            e.clone()
                .mut_ratios(
                    r![
                    (3, 2, 0.0, 0.6, 1.0)
                    (5, 4, 5.0, 0.6, -1.0)
                ]),
            e.clone()
                .mut_ratios(
                    r![
                    (5, 4, -10.0, 0.6, -1.0)
                    (1, 2, -1.0, 0.6, 1.0)
                ]),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![
        phrase1.clone(),
        phrase1.clone(),
        phrase1.clone(),
        phrase1.clone(),
        phrase1.clone(),
        phrase1.clone(),
        phrase1.clone(),
        end
    ].render(&mut oscillator)
}
