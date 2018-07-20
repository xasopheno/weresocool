use ratios::{mono_ratios, StereoRatios, R};

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub frequency: f32,
    pub ratios: StereoRatios,
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
    pub fn new(frequency: f32, ratios: StereoRatios, length: f32, gain: f32) -> Event {
        Event {
            frequency,
            ratios,
            length,
            gain,
        }
    }
}

impl Phrase {
    pub fn phrase_from_vec(mut events: Vec<Event>) -> Phrase {
        Phrase { events }
    }
}

pub trait Render<T> {
    fn collapse_to_vec_events(&mut self) -> Vec<Event>;
}

impl Render<Phrase> for Phrase {
    fn collapse_to_vec_events(&mut self) -> Vec<Event> {
        self.events.clone()
    }
}

impl Render<Vec<Phrase>> for Vec<Phrase> {
    fn collapse_to_vec_events(&mut self) -> Vec<Event> {
        let mut vec_events: Vec<Event> = vec![];
        for phrase in self.iter_mut() {
            vec_events.append(&mut phrase.events)
        }
        vec_events
    }
}

pub trait Mutate<T> {
    fn transpose(&mut self, mul: f32, add: f32) -> T;
    fn mut_ratios(&mut self, ratios: StereoRatios) -> T;
    fn mut_length(&mut self, mul: f32, add: f32) -> T;
    fn mut_gain(&mut self, mul: f32, add: f32) -> T;
}

impl Mutate<Event> for Event {
    fn transpose(&mut self, mul: f32, add: f32) -> Event {
        self.frequency = self.frequency * mul + add;
        self.clone()
    }

    fn mut_ratios(&mut self, ratios: StereoRatios) -> Event {
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

    fn mut_ratios(&mut self, ratios: StereoRatios) -> Phrase {
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

pub fn generate_test_phrase() -> Vec<Event> {
    let e = Event::new(70.0, mono_ratios(), 1.0, 1.0);
    let mut phrase1 = Phrase {
        events: vec![
            e.clone(),
            e.clone().transpose(6.0 / 5.0, 0.0),
            e.clone().transpose(7.0 / 4.0, 0.0),
        ],
    };

    vec![phrase1.clone()].collapse_to_vec_events()
}
//
//pub fn generate_pop() -> Vec<Event> {
//    let e = Event::new(50.0, simple_ratios(), 1.0, 1.0);
//    let mut phrase1 = Phrase {
//        events: vec![e.clone()],
//    };
//
//    let phrase2 = phrase1.clone().transpose(2.0 / 3.0, 10.0);
//    //        .mut_length(2.0, 1.0);
//    //        .mut_gain(0.9, 0.0);
//
//    vec![
//        phrase1.clone(),
//        phrase2.clone(),
//        //        phrase1.clone(),
//        //        phrase4.clone(),
//        //        phrase2.clone(),
//        //        phrase3.clone(),
//    ].collapse_to_vec_events()
//}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_mutate_event() {
        let result = Event::new(100.0, mono_ratios(), 1.0, 1.0)
            .mut_ratios(mono_ratios())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Event {
            frequency: 150.0,
            ratios: mono_ratios(),
            length: 3.0,
            gain: 0.9,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mutate_phrase() {
        let mut phrase = Phrase {
            events: vec![
                Event::new(100.0, mono_ratios(), 1.0, 1.0),
                Event::new(50.0, mono_ratios(), 2.0, 1.0),
            ],
        };

        let result = phrase
            .mut_ratios(mono_ratios())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Phrase {
            events: vec![
                Event::new(150.0, mono_ratios(), 3.0, 0.9),
                Event::new(75.0, mono_ratios(), 5.0, 0.9),
            ],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collapse_phrase() {
        let mut phrase1 = Phrase {
            events: vec![
                Event::new(100.0, mono_ratios(), 1.0, 1.0),
                Event::new(50.0, mono_ratios(), 2.0, 1.0),
            ],
        };

        let phrase2 = phrase1
            .clone()
            .mut_ratios(mono_ratios())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let result = vec![phrase1, phrase2].collapse_to_vec_events();

        let expected = vec![
            Event::new(100.0, mono_ratios(), 1.0, 1.0),
            Event::new(50.0, mono_ratios(), 2.0, 1.0),
            Event::new(150.0, mono_ratios(), 3.0, 0.9),
            Event::new(75.0, mono_ratios(), 5.0, 0.9),
        ];
        assert_eq!(result, expected);
    }
}
