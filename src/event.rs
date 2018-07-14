use ratios::{R, StereoRatios};

#[derive(Debug, Clone)]
pub struct Event {
    pub frequency: f32,
    pub ratios: StereoRatios,
    pub length: f32,
    pub gain: f32,
}

#[derive(Debug, Clone)]
pub struct Phrase {
    pub events: Vec<Event>,
}

enum Mutable {
    Event,
    Phrase,
}

impl Event {
    pub fn new(frequency: f32, ratios: StereoRatios, length: f32, gain: f32) -> Event {
        Event {
            frequency,
            ratios,
            length,
            gain
        }
    }
}

impl Phrase {
    pub fn phrase_from_vec(mut events: Vec<Event>) -> Phrase {
        Phrase {
            events
        }
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use ratios::{simple_ratios, mono_ratios};

    #[test]
    fn test_mutate_event() {
        let result = Event::new(100.0, simple_ratios(), 1.0, 1.0)
            .mut_ratios(mono_ratios())
            .transpose(3.0/2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Event {
            frequency: 150.0,
            ratios: mono_ratios(),
            length: 3.0,
            gain: 0.9,
        };
        assert_eq!(result.frequency, expected.frequency);
        assert_eq!(result.ratios, expected.ratios);
        assert_eq!(result.length, expected.length);
        assert_eq!(result.gain, expected.gain);
    }
}
