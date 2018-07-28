use ratios::{simple_ratios, R, Pan};
use new_oscillator::{NewOscillator, StereoWaveform};
use settings::{get_default_app_settings};

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub frequency: f32,
    pub ratios: Vec<R>,
    pub length: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Phrase {
    pub events: Vec<Event>,
}

pub enum Mutable {
    Event,
    Phrase,
}

impl Event {
    pub fn new(frequency: f32, ratios: Vec<R>, length: f32, gain: f32) -> Event {
        Event {
            frequency,
            ratios,
            length,
            gain,
        }
    }
}

impl Phrase {
    pub fn phrase_from_vec(events: Vec<Event>) -> Phrase {
        Phrase { events }
    }
}

pub trait Render<T> {
    fn render(&mut self, oscillator: &mut NewOscillator) -> StereoWaveform;
}

impl Render<Event> for Event {
    fn render(&mut self, oscillator: &mut NewOscillator) -> StereoWaveform {
        oscillator.update_freq_gain_and_ratios(self.frequency, self.gain, &self.ratios);
        let n_samples_to_generate = (self.length * 44_100.0).floor() as usize;
        oscillator.generate(n_samples_to_generate)
    }
}

impl Render<Phrase> for Phrase {
    fn render(&mut self, oscillator: &mut NewOscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        for mut event in self.events.clone() {
            let stereo_waveform = event.render(oscillator);
            result.append(stereo_waveform);
        };

        result
    }
}

impl Render<Vec<Phrase>> for Vec<Phrase> {
    fn render(&mut self, oscillator: &mut NewOscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut vec_events: Vec<Event> = vec![];
        for phrase in self.iter_mut() {
            let stereo_waveform = phrase.render(oscillator);
            result.append(stereo_waveform);
        }

        result
    }
}

pub trait Mutate<T> {
    fn transpose(&mut self, mul: f32, add: f32) -> T;
    fn mut_ratios(&mut self, ratios: Vec<R>) -> T;
    fn mut_length(&mut self, mul: f32, add: f32) -> T;
    fn mut_gain(&mut self, mul: f32, add: f32) -> T;
}

impl Mutate<Event> for Event {
    fn transpose(&mut self, mul: f32, add: f32) -> Event {
        self.frequency = self.frequency * mul + add;
        self.clone()
    }

    fn mut_ratios(&mut self, ratios: Vec<R>) -> Event {
        self.ratios = ratios;
        self.clone()
    }

    fn mut_length(&mut self, mul: f32, add: f32) -> Event {
        self.length = self.length * mul + add;
        self.clone()
    }

    fn mut_gain(&mut self, mul: f32, add: f32) -> Event {
        self.gain = self.gain * mul + add;
        self.clone()
    }
}

impl Mutate<Phrase> for Phrase {
    fn transpose(&mut self, mul: f32, add: f32) -> Phrase {
        for event in self.events.iter_mut() {
            event.transpose(mul, add);
        }
        self.clone()
    }

    fn mut_ratios(&mut self, ratios: Vec<R>) -> Phrase {
        for event in self.events.iter_mut() {
            event.mut_ratios(ratios.clone());
        }
        self.clone()
    }

    fn mut_length(&mut self, mul: f32, add: f32) -> Phrase {
        for event in self.events.iter_mut() {
            event.mut_length(mul, add);
        }
        self.clone()
    }

    fn mut_gain(&mut self, mul: f32, add: f32) -> Phrase {
        for event in self.events.iter_mut() {
            event.mut_gain(mul, add);
        }
        self.clone()
    }
}

pub fn generate_test_phrase() -> StereoWaveform {
    let settings = get_default_app_settings();
    let r = vec![
        R::atio(1, 2, 0.0, 0.1, Pan::Right),
        R::atio(1, 2, 2.0, 0.1, Pan::Right),
        R::atio(1, 2, 0.0, 0.1, Pan::Left),
        R::atio(1, 2, 2.0, 0.1, Pan::Left),
//
        R::atio(1, 1, -1.0, 0.8, Pan::Right),
        R::atio(1, 1, 0.0, 0.8, Pan::Right),
        R::atio(2, 1, 0.0, 0.8, Pan::Right),
//
        R::atio(1, 1, 1.0, 0.8, Pan::Left),
        R::atio(1, 1, 3.0, 0.8, Pan::Left),
        R::atio(2, 1, 0.0, 0.8, Pan::Left),
//
        R::atio(11, 1, 13.0, 0.02, Pan::Right),
        R::atio(11, 1, 0.0, 0.02, Pan::Right),
        R::atio(11, 1, 13.0, 0.02, Pan::Left),
        R::atio(11, 1, 0.0, 0.02, Pan::Left),
//
        R::atio(15, 1, 0.0, 0.02, Pan::Right),
        R::atio(17, 1, 0.0, 0.02, Pan::Left),


    ];
    let mut oscillator = NewOscillator::init(r.clone(), &settings);
    let freq = 150.0;
    let e = Event::new(freq, r.clone(), 0.95, 1.0);
    let phrase1 = Phrase {
        events: vec![
            e.clone(),
            e.clone()
                .mut_ratios(vec![
                    R::atio(1, 2, 0.0, 0.1, Pan::Right),
                    R::atio(1, 2, 2.0, 0.1, Pan::Right),
                    R::atio(1, 2, 0.0, 0.1, Pan::Left),
                    R::atio(1, 2, 3.0, 0.1, Pan::Left),
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
            e.clone()
                .mut_ratios(vec![
                    R::atio(1, 2, 0.0, 0.1, Pan::Right),
                    R::atio(1, 2, 2.0, 0.1, Pan::Right),
                    R::atio(1, 2, 0.0, 0.1, Pan::Left),
                    R::atio(1, 2, 3.0, 0.1, Pan::Left),
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
            e.clone()
                .mut_ratios(vec![
                    R::atio(1, 2, 0.0, 0.1, Pan::Right),
                    R::atio(1, 2, 3.0, 0.1, Pan::Right),
                    R::atio(1, 2, 0.0, 0.1, Pan::Left),
                    R::atio(1, 2, 2.0, 0.1, Pan::Left),
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
                    R::atio(17, 1, 0.0, 0.015, Pan::Right),
                    R::atio(15, 1, 0.0, 0.015, Pan::Left),
                ]),

            e.clone()
                .mut_length(2.0, 0.0)
                .mut_ratios(vec![
                    R::atio(1, 2, 0.0, 0.1, Pan::Right),
                    R::atio(1, 2, 2.0, 0.1, Pan::Right),
                    R::atio(1, 2, 0.0, 0.1, Pan::Left),
                    R::atio(1, 2, 3.0, 0.1, Pan::Left),
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
            e.clone()
                .mut_ratios(vec![
                    R::atio(1, 2, 0.0, 0.1, Pan::Right),
                    R::atio(1, 2, 3.0, 0.1, Pan::Right),
                    R::atio(1, 2, 0.0, 0.1, Pan::Left),
                    R::atio(1, 2, 2.0, 0.1, Pan::Left),
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
                ])
        ],
    };
    let mut phrase2 = phrase1.clone().transpose(4.0 / 3.0, 0.0);
    phrase2.events[2].mut_length(3.0, 0.0);

    let resolution = Phrase {
        events: vec![e.clone()]
    };

    let end = Phrase {
        events: vec![Event::new(0.0, r.clone(), 3.0, 0.0)]
    };

    vec![
        phrase1.clone(),
        phrase2.clone(),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(4.0/5.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(2.0/3.0, 0.0),

        phrase1.clone()
            .mut_ratios(r.clone()),
        phrase2.clone(),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(4.0/5.0, 0.0),
        phrase2
            .clone()
            .mut_length(0.25, 0.0)
            .transpose(2.0/3.0, 0.0),

        resolution,
        end
    ].render(&mut oscillator)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_mutate_event() {
        let result = Event::new(100.0, simple_ratios(), 1.0, 1.0)
            .mut_ratios(simple_ratios())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Event {
            frequency: 150.0,
            ratios: simple_ratios(),
            length: 3.0,
            gain: 0.9,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mutate_phrase() {
        let mut phrase = Phrase {
            events: vec![
                Event::new(100.0, simple_ratios(), 1.0, 1.0),
                Event::new(50.0, simple_ratios(), 2.0, 1.0),
            ],
        };

        let result = phrase
            .mut_ratios(simple_ratios())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Phrase {
            events: vec![
                Event::new(150.0, simple_ratios(), 3.0, 0.9),
                Event::new(75.0, simple_ratios(), 5.0, 0.9),
            ],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collapse_phrase() {
        let mut phrase1 = Phrase {
            events: vec![
                Event::new(100.0, simple_ratios(), 1.0, 1.0),
                Event::new(50.0, simple_ratios(), 2.0, 1.0),
            ],
        };

        let phrase2 = phrase1
            .clone()
            .mut_ratios(simple_ratios())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let result = vec![phrase1, phrase2].collapse_to_vec_events();

        let expected = vec![
            Event::new(100.0, simple_ratios(), 1.0, 1.0),
            Event::new(50.0, simple_ratios(), 2.0, 1.0),
            Event::new(150.0, simple_ratios(), 3.0, 0.9),
            Event::new(75.0, simple_ratios(), 5.0, 0.9),
        ];
        assert_eq!(result, expected);
    }
}
