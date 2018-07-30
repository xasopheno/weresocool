use ratios::{R};
use oscillator::{NewOscillator, StereoWaveform};

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


#[cfg(test)]
pub mod tests {
    use super::*;
    use ratios::simple_ratios;

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
