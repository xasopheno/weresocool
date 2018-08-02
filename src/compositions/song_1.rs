use event::{Event, Mutate, Phrase, Render};
use instrument::{
    oscillator::Oscillator,
    stereo_waveform::StereoWaveform
};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = vec![
        R::atio(1, 2, 0.0, 0.22, Pan::Right),
        R::atio(1, 2, 2.0, 0.22, Pan::Right),
        R::atio(1, 2, 0.0, 0.22, Pan::Left),
        R::atio(1, 2, 2.0, 0.22, Pan::Left),
        //
        R::atio(1, 1, -1.0, 0.8, Pan::Right),
        R::atio(1, 1, 0.0, 0.8, Pan::Right),
        R::atio(2, 1, 0.0, 0.8, Pan::Right),
        //
        R::atio(1, 1, 1.0, 0.8, Pan::Left),
        R::atio(1, 1, 3.0, 0.8, Pan::Left),
        R::atio(2, 1, 0.0, 0.8, Pan::Left),
        //s
        R::atio(11, 1, 13.0, 0.02, Pan::Right),
        R::atio(11, 1, 0.0, 0.02, Pan::Right),
        R::atio(11, 1, 13.0, 0.02, Pan::Left),
        R::atio(11, 1, 0.0, 0.02, Pan::Left),
        //
        R::atio(15, 1, 0.0, 0.02, Pan::Right),
        R::atio(17, 1, 0.0, 0.02, Pan::Left),
    ];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 150.0;
    let e = Event::new(freq, r.clone(), 1.2, 1.0);
    let phrase1 = Phrase {
        events: vec![
            e.clone(),
            e.clone().mut_ratios(vec![
                R::atio(1, 2, 0.0, 0.22, Pan::Right),
                R::atio(1, 2, -2.0, 0.22, Pan::Right),
                R::atio(1, 2, 0.0, 0.22, Pan::Left),
                R::atio(1, 2, 3.0, 0.22, Pan::Left),
                //
                R::atio(1, 1, 0.0, 0.8, Pan::Right),
                R::atio(3, 2, 0.0, 0.8, Pan::Right),
                R::atio(5, 2, 0.0, 0.8, Pan::Right),
                //
                R::atio(5, 4, 3.0, 0.8, Pan::Left),
                R::atio(5, 4, 3.0, 0.8, Pan::Left),
                R::atio(3, 1, 3.0, 0.8, Pan::Left),
                //
                R::atio(11, 1, 11.0, 0.02, Pan::Right),
                R::atio(11, 1, 0.0, 0.02, Pan::Right),
                R::atio(11, 1, 11.0, 0.02, Pan::Left),
                R::atio(11, 1, 0.0, 0.02, Pan::Left),
                //
                R::atio(15, 1, 0.0, 0.015, Pan::Right),
                R::atio(17, 1, 0.0, 0.015, Pan::Left),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 2, 0.0, 0.22, Pan::Right),
                R::atio(1, 2, -2.0, 0.22, Pan::Right),
                R::atio(1, 2, 0.0, 0.22, Pan::Left),
                R::atio(1, 2, 3.0, 0.22, Pan::Left),
                //
                R::atio(5, 3, 0.0, 0.8, Pan::Right),
                R::atio(5, 3, 0.0, 0.8, Pan::Right),
                R::atio(5, 2, 0.0, 0.8, Pan::Right),
                //
                R::atio(4, 3, 3.0, 0.8, Pan::Left),
                R::atio(1, 1, 3.0, 0.8, Pan::Left),
                R::atio(3, 1, 3.0, 0.8, Pan::Left),
                //
                R::atio(13, 1, 9.0, 0.02, Pan::Right),
                R::atio(13, 1, 0.0, 0.02, Pan::Right),
                R::atio(13, 1, 9.0, 0.02, Pan::Left),
                R::atio(13, 1, 0.0, 0.02, Pan::Left),
                //
                R::atio(14, 1, 0.0, 0.015, Pan::Right),
                R::atio(15, 1, 0.0, 0.015, Pan::Left),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 2, 0.0, 0.22, Pan::Right),
                R::atio(1, 2, 3.0, 0.22, Pan::Right),
                R::atio(1, 2, 0.0, 0.22, Pan::Left),
                R::atio(1, 2, 2.0, 0.22, Pan::Left),
                //
                R::atio(3, 2, 0.0, 0.8, Pan::Right),
                R::atio(3, 2, 13.0, 0.8, Pan::Right),
                R::atio(9, 4, 0.0, 0.8, Pan::Right),
                //
                R::atio(9, 8, 0.0, 0.8, Pan::Left),
                R::atio(9, 8, 6.0, 0.8, Pan::Left),
                R::atio(12, 4, 0.0, 0.8, Pan::Left),
                //
                R::atio(13, 1, 3.0, 0.02, Pan::Right),
                R::atio(13, 1, 5.0, 0.02, Pan::Right),
                R::atio(13, 1, 7.0, 0.02, Pan::Left),
                R::atio(13, 1, 9.0, 0.02, Pan::Left),
                //
                R::atio(15, 1, 0.0, 0.015, Pan::Right),
                R::atio(17, 1, 0.0, 0.015, Pan::Left),
            ]),
            e.clone().mut_length(2.0, 0.0).mut_ratios(vec![
                R::atio(1, 2, 0.0, 0.22, Pan::Right),
                R::atio(1, 2, 2.0, 0.22, Pan::Right),
                R::atio(1, 2, 0.0, 0.22, Pan::Left),
                R::atio(1, 2, 3.0, 0.22, Pan::Left),
                //
                R::atio(7, 4, 0.0, 0.8, Pan::Right),
                R::atio(7, 2, 0.0, 0.8, Pan::Right),
                R::atio(7, 2, 1.0, 0.8, Pan::Right),
                //
                R::atio(7, 6, 0.0, 0.8, Pan::Left),
                R::atio(7, 6, 9.0, 0.8, Pan::Left),
                R::atio(7, 3, 3.0, 0.8, Pan::Left),
                //
                R::atio(13, 1, 11.0, 0.02, Pan::Right),
                R::atio(13, 1, 9.0, 0.02, Pan::Right),
                R::atio(13, 1, 7.0, 0.02, Pan::Left),
                R::atio(13, 1, 1.0, 0.02, Pan::Left),
                //
                R::atio(15, 1, 0.0, 0.015, Pan::Right),
                R::atio(17, 1, 0.0, 0.015, Pan::Left),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 2, 0.0, 0.22, Pan::Right),
                R::atio(1, 2, 3.0, 0.22, Pan::Right),
                R::atio(1, 2, 0.0, 0.22, Pan::Left),
                R::atio(1, 2, 2.0, 0.22, Pan::Left),
                //
                R::atio(10, 4, 0.0, 0.8, Pan::Right),
                R::atio(3, 2, 0.0, 0.8, Pan::Right),
                R::atio(3, 2, 1.0, 0.8, Pan::Right),
                //
                R::atio(15, 16, 3.0, 0.8, Pan::Left),
                R::atio(15, 8, 3.0, 0.8, Pan::Left),
                R::atio(15, 8, 2.0, 0.8, Pan::Left),
                //
                R::atio(13, 1, 8.0, 0.02, Pan::Right),
                R::atio(13, 1, 0.0, 0.02, Pan::Right),
                R::atio(13, 1, 0.0, 0.02, Pan::Left),
                R::atio(13, 1, 7.0, 0.02, Pan::Left),
                R::atio(19, 1, 0.0, 0.015, Pan::Right),
                R::atio(18, 1, 0.0, 0.015, Pan::Left),
            ]),
        ],
    };
    let mut phrase2 = phrase1.clone().transpose(4.0 / 3.0, 0.0);
    phrase2.events[2].mut_length(3.0, 0.0);

    fn resolution() -> Phrase {
        let r = vec![
            R::atio(1, 2, 0.0, 0.22, Pan::Right),
            R::atio(1, 2, -3.0, 0.22, Pan::Right),
            R::atio(1, 2, 0.0, 0.22, Pan::Left),
            R::atio(1, 2, 4.0, 0.22, Pan::Left),
            //
            R::atio(1, 1, -1.0, 0.8, Pan::Right),
            R::atio(3, 2, 0.0, 0.8, Pan::Right),
            R::atio(3, 2, 4.2, 0.8, Pan::Right),
            //
            R::atio(1, 1, 1.0, 0.8, Pan::Left),
            R::atio(5, 4, 3.0, 0.8, Pan::Left),
            R::atio(2, 1, 0.6, 0.8, Pan::Left),
            //
            R::atio(11, 1, 17.0, 0.02, Pan::Right),
            R::atio(12, 1, 0.0, 0.02, Pan::Right),
            R::atio(13, 1, 15.0, 0.02, Pan::Left),
            R::atio(14, 1, 0.0, 0.02, Pan::Left),
            //
            R::atio(17, 1, -11.0, 0.02, Pan::Right),
            R::atio(15, 1, -10.0, 0.02, Pan::Left),
        ];
        let e = Event::new(100.0, r.clone(), 3.0, 1.0);
        Phrase {
            events: vec![
                e.clone().transpose(3.0 / 4.0, 0.0).mut_length(0.7, 0.0),
                e.clone(),
            ],
        }
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![
        phrase1.clone(),
        phrase2.clone(),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(4.0 / 5.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(2.0 / 3.0, 0.0),
        phrase1.clone().mut_ratios(r.clone()),
        phrase2.clone(),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(4.0 / 5.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(2.0 / 3.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(3.0 / 4.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(4.0 / 5.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(2.0 / 3.0, 0.0),
        resolution(),
        end,
    ].render(&mut oscillator)
}
