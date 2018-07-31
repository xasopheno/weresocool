use event::{Event, Mutate, Phrase, Render};
use oscillator::{NewOscillator, StereoWaveform};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = vec![
        R::atio(0, 1, 0.0, 0.5, Pan::Left),
        R::atio(0, 1, 0.0, 0.5, Pan::Left),
        //
        R::atio(0, 1, 0.0, 0.5, Pan::Right),
        R::atio(0, 1, 0.0, 0.5, Pan::Right),
        //
        R::atio(0, 1, 0.0, 0.2, Pan::Left),
        R::atio(0, 1, 0.0, 0.2, Pan::Right),
    ];
    let mut oscillator = NewOscillator::init(r.clone(), &settings);
    let freq = 230.0;
    let e = Event::new(freq, r.clone(), 2.5, 1.0);
    let phrase1 = Phrase {
        events: vec![
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 11.0, 0.5, Pan::Left),
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(1, 1, 11.0, 0.5, Pan::Right),
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![phrase1.clone(), phrase1.clone(), end].render(&mut oscillator)
}
