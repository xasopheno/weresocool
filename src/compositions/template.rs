use event::{Event, Mutate, Phrase, Render};
use instrument::oscillator::{Oscillator, StereoWaveform};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = r![(1, 1, 0.0, 0.0, 0.0), (1, 1, 0.0, 0.0, 0.0)];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 230.0;
    let e = Event::new(freq, r.clone(), 0.5, 0.8);
    let phrase1 = Phrase {
        events: vec![
            e.clone()
                .mut_length(3.5, 0.0)
                .mut_ratios(r![(1, 1, 0.0, 0.6, 0.5), (1, 1, -2.0, 0.6, 0.5)]),
            e.clone()
                .mut_length(3.5, 0.0)
                .mut_ratios(r![(1, 1, 0.0, 0.6, -1.0), (1, 1, -3.0, 0.6, 1.0)]),
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
        end,
    ].render(&mut oscillator)
}
