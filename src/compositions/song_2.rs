use event::{Mutate, Event, Phrase, Render};
use ratios::{R, Pan};
use oscillator::{StereoWaveform, NewOscillator};
use settings::{get_default_app_settings};

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = vec![
        R::atio(1, 1, 0.0, 0.8, Pan::Left),
        R::atio(3, 2, 0.0, 0.0, Pan::Left),
//
        R::atio(1, 1, -1.0, 0.8, Pan::Right),
        R::atio(3, 2, -1.0, 0.0, Pan::Right),
    ];
    let mut oscillator = NewOscillator::init(r.clone(), &settings);
    let freq = 150.0;
    let e = Event::new(freq, r.clone(), 3.0, 1.0);
    let phrase1 = Phrase {
        events: vec![
            e.clone(),
            e.clone()
                .mut_ratios(vec![
                    R::atio(1, 1, 0.0, 0.8, Pan::Left),
                    R::atio(3, 2, 0.0, 0.8, Pan::Left),

                    R::atio(1, 1, -1.0, 0.8, Pan::Right),
                    R::atio(3, 2, -1.0, 0.8, Pan::Right),
                ]),
            e.clone()
                .mut_ratios(vec![
                    R::atio(15, 8, 0.0, 0.8, Pan::Left),
                    R::atio(9, 8, 0.0, 0.8, Pan::Left),
//
                    R::atio(11, 4, -9.0, 0.8, Pan::Right),
                    R::atio(11, 8, -1.0, 0.8, Pan::Right),
                ])
        ],
    };

    let end = Phrase {
        events: vec![Event::new(150.0, vec![
            R::atio(1, 1, 0.0, 0.0, Pan::Left),
            R::atio(3, 2, 0.0, 0.0, Pan::Left),

            R::atio(1, 1, -1.0, 0.0, Pan::Right),
            R::atio(3, 2, -1.0, 0.0, Pan::Right),
        ], 3.0, 0.0)]
    };

    vec![
        phrase1.clone(),
//        end
    ].render(&mut oscillator)
}
