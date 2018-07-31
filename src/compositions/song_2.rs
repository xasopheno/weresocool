use event::{Event, Mutate, Phrase, Render};
use oscillator::{Oscillator, StereoWaveform};
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
    let mut oscillator = Oscillator::init(r.clone(), &settings);
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
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 3.0, 0.5, Pan::Left),
                R::atio(1, 1, 3.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 9.0, 0.5, Pan::Right),
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(9, 8, 6.0, 0.5, Pan::Left),
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 3.0, 0.5, Pan::Right),
                R::atio(5, 4, 5.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(5, 4, -10.0, 0.5, Pan::Left),
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 3.0, 0.5, Pan::Right),
                R::atio(9, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(1, 1, 9.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                R::atio(1, 1, 1.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(1, 1, 5.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                R::atio(1, 1, 4.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(1, 1, 2.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                R::atio(1, 1, 7.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(3, 2, -10.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 1.0, 0.5, Pan::Right),
                R::atio(1, 1, -4.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 3.0, 0.5, Pan::Left),
                R::atio(1, 1, 3.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 1.0, 0.5, Pan::Right),
                R::atio(3, 2, 7.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, -1.0, 0.5, Pan::Left),
                R::atio(4, 3, 3.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 1.0, 0.5, Pan::Right),
                R::atio(3, 2, 0.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(3, 2, 2.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                R::atio(9, 8, 5.0, 0.5, Pan::Right),
                //
                R::atio(0, 1, 0.0, 0.2, Pan::Left),
                R::atio(0, 1, 0.0, 0.2, Pan::Right),
            ]),
            ////////////////////////////////
            e.clone().mut_ratios(vec![
                R::atio(4, 5, 0.0, 0.5, Pan::Left),
                R::atio(9, 8, 0.0, 0.5, Pan::Left),
                //
                R::atio(1, 2, 1.0, 0.5, Pan::Right),
                R::atio(4, 3, 0.0, 0.5, Pan::Right),
                //
                R::atio(9, 4, 1.0, 0.2, Pan::Left),
                R::atio(9, 4, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(5, 4, 0.0, 0.5, Pan::Left),
                R::atio(7, 6, 0.0, 0.5, Pan::Left),
                //
                R::atio(5, 6, 1.0, 0.5, Pan::Right),
                R::atio(15, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(12, 4, 0.0, 0.2, Pan::Left),
                R::atio(12, 4, 2.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(9, 8, 0.0, 0.5, Pan::Left),
                R::atio(3, 4, 0.0, 0.5, Pan::Left),
                //
                R::atio(5, 3, 1.0, 0.5, Pan::Right),
                R::atio(11, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(10, 4, 3.0, 0.2, Pan::Left),
                R::atio(10, 4, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(5, 4, 0.0, 0.5, Pan::Left),
                //
                R::atio(7, 8, 7.0, 0.5, Pan::Right),
                R::atio(7, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(3, 2, 4.0, 0.2, Pan::Left),
                R::atio(3, 2, 0.0, 0.2, Pan::Right),
            ]),
            e.clone().transpose(4.0 / 3.0, 0.0).mut_ratios(vec![
                R::atio(3, 2, 0.0, 0.5, Pan::Left),
                R::atio(3, 4, 5.0, 0.5, Pan::Left),
                //
                R::atio(1, 1, 0.0, 0.5, Pan::Right),
                R::atio(5, 4, 0.0, 0.5, Pan::Right),
                //
                R::atio(9, 8, 0.0, 0.2, Pan::Left),
                R::atio(9, 8, 1.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(7, 6, 0.0, 0.5, Pan::Left),
                R::atio(5, 4, 11.0, 0.5, Pan::Left),
                //
                R::atio(6, 5, 0.0, 0.5, Pan::Right),
                R::atio(9, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(9, 8, 0.0, 0.2, Pan::Left),
                R::atio(9, 8, 3.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_ratios(vec![
                R::atio(1, 1, 0.0, 0.5, Pan::Left),
                R::atio(5, 4, 1.0, 0.5, Pan::Left),
                //
                R::atio(3, 4, 3.0, 0.5, Pan::Right),
                R::atio(3, 2, 0.0, 0.5, Pan::Right),
                //
                R::atio(9, 8, 0.0, 0.2, Pan::Left),
                R::atio(9, 8, 5.0, 0.2, Pan::Right),
            ]),
            e.clone().transpose(7.0 / 8.0, 0.0).mut_ratios(vec![
                R::atio(7, 6, 0.0, 0.5, Pan::Left),
                R::atio(5, 4, 11.0, 0.5, Pan::Left),
                //
                R::atio(6, 5, 0.0, 0.5, Pan::Right),
                R::atio(9, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(6, 10, 5.0, 0.2, Pan::Left),
                R::atio(6, 10, -4.0, 0.2, Pan::Right),
            ]),
            e.clone().mut_length(1.5, 0.0).mut_ratios(vec![
                R::atio(7, 3, 0.0, 0.5, Pan::Left),
                R::atio(8, 9, 11.0, 0.5, Pan::Left),
                //
                R::atio(11, 8, 0.0, 0.5, Pan::Right),
                R::atio(5, 4, 0.0, 0.5, Pan::Right),
                //
                R::atio(1, 2, 5.0, 0.2, Pan::Left),
                R::atio(1, 2, -4.0, 0.2, Pan::Right),
            ]),
            e.clone()
                .transpose(9.0 / 8.0, 0.0)
                .mut_length(0.5, 0.0)
                .mut_ratios(vec![
                    R::atio(7, 4, 0.0, 0.5, Pan::Left),
                    R::atio(8, 10, 11.0, 0.5, Pan::Left),
                    //
                    R::atio(11, 8, 0.0, 0.5, Pan::Right),
                    R::atio(5, 4, 5.0, 0.5, Pan::Right),
                    //
                    R::atio(1, 2, 5.0, 0.2, Pan::Left),
                    R::atio(1, 3, 0.0, 0.2, Pan::Right),
                ]),
            e.clone()
                .transpose(9.0 / 8.0, 0.0)
                .mut_length(0.5, 0.0)
                .mut_ratios(vec![
                    R::atio(5, 4, 0.0, 0.5, Pan::Left),
                    R::atio(8, 7, 1.0, 0.5, Pan::Left),
                    //
                    R::atio(10, 4, 0.0, 0.5, Pan::Right),
                    R::atio(6, 5, 5.0, 0.5, Pan::Right),
                    //
                    R::atio(1, 3, 5.0, 0.2, Pan::Left),
                    R::atio(1, 2, 0.0, 0.2, Pan::Right),
                ]),
            e.clone()
                .transpose(5.0 / 4.0, 0.0)
                .mut_length(0.5, 0.0)
                .mut_ratios(vec![
                    R::atio(7, 4, 0.0, 0.5, Pan::Left),
                    R::atio(8, 9, 1.0, 0.5, Pan::Left),
                    //
                    R::atio(11, 8, 0.0, 0.5, Pan::Right),
                    R::atio(6, 5, 5.0, 0.5, Pan::Right),
                    //
                    R::atio(1, 2, 5.0, 0.2, Pan::Left),
                    R::atio(1, 3, -5.0, 0.2, Pan::Right),
                ]),
            e.clone().mut_length(0.5, 0.0).mut_ratios(vec![
                R::atio(11, 9, 0.0, 0.5, Pan::Left),
                R::atio(10, 9, 1.0, 0.5, Pan::Left),
                //
                R::atio(11, 8, 0.0, 0.5, Pan::Right),
                R::atio(10, 8, 5.0, 0.5, Pan::Right),
                //
                R::atio(1, 2, 5.0, 0.2, Pan::Left),
                R::atio(1, 3, -5.0, 0.2, Pan::Right),
            ]),
            e.clone()
                .transpose(7.0 / 8.0, 0.0)
                .mut_length(0.50, 0.0)
                .mut_ratios(vec![
                    R::atio(7, 6, 0.0, 0.5, Pan::Left),
                    R::atio(5, 4, 11.0, 0.5, Pan::Left),
                    //
                    R::atio(6, 5, 0.0, 0.5, Pan::Right),
                    R::atio(9, 8, 0.0, 0.5, Pan::Right),
                    //
                    R::atio(6, 10, 5.0, 0.2, Pan::Left),
                    R::atio(6, 10, -4.0, 0.2, Pan::Right),
                ]),
            e.clone().mut_length(0.95, 0.0).mut_ratios(vec![
                R::atio(4, 3, 5.0, 0.5, Pan::Left),
                R::atio(4, 3, -6.0, 0.5, Pan::Left),
                //
                R::atio(7, 6, 0.0, 0.5, Pan::Right),
                R::atio(3, 8, 0.0, 0.5, Pan::Right),
                //
                R::atio(3, 4, 5.0, 0.2, Pan::Left),
                R::atio(3, 4, -4.0, 0.2, Pan::Right),
            ]),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    vec![phrase1.clone(), phrase1.clone(), end].render(&mut oscillator)
}
