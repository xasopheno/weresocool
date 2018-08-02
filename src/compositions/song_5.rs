use event::{Event, Mutate, Phrase, Render};
use instrument::{
    oscillator::Oscillator,
    stereo_waveform::StereoWaveform
};
use ratios::{Pan, R};
use settings::get_default_app_settings;

pub fn generate_composition() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = r![
            (1, 1, 1.0, 0.6, 0.5),
            (1, 1, 2.0, 0.6, 0.5),
            (1, 1, 3.0, 0.6, 0.5),
            (1, 1, -2.0, 0.6, 0.5)
        ];
    let mut oscillator = Oscillator::init(r.clone(), &settings);
    let freq = 200.0;
    let e = Event::new(freq, r.clone(), 0.25, 0.8);
    let phrase1 = Phrase {
        events: vec![
            e.clone()
                .transpose(8.0/5.0, 0.0),
            e.clone()
                .transpose(4.0/3.0, 0.0),
            e.clone(),
            e.clone()
                .transpose(7.0/8.0, 0.0),
            e.clone()
                .transpose(3.0/4.0, 0.0),
            e.clone()
                .transpose(5.0/3.0, 0.0),
            e.clone()
                .transpose(1.0/2.0, 0.0),
            e.clone()
                .transpose(2.0/3.0, 0.0),
            e.clone()
                .transpose(5.0/4.0, 0.0),
            e.clone()
                .transpose(9.0/8.0, 0.0),
            e.clone()
                .mut_length(5.0, 0.0)
                .transpose(5.0/4.0, 0.0),
            e.clone()
                .mut_length(6.0, 0.0)
                .transpose(0.0/1.0, 0.0),
        ],
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)],
    };

    let r2 = r![
            (3, 2, 1.0, 0.6, 0.5),
            (1, 1, 2.0, 0.6, 0.5),
            (1, 1, 3.0, 0.6, 0.5),
            (2, 1, -2.0, 0.6, 0.5)
        ];
    vec![
        phrase1.clone(),
        phrase1.clone()
            .mut_ratios(r2.clone()),
        phrase1.clone(),
        phrase1.clone()
            .mut_ratios(r2),
//        phrase1.clone(),
//        phrase1.clone(),
        end,
    ].render(&mut oscillator)
}
