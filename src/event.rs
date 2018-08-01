use oscillator::{Oscillator, StereoWaveform};
use ratios::R;

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
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform;
}

impl Render<Event> for Event {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        oscillator.update_freq_gain_and_ratios(self.frequency, self.gain, &self.ratios);
        let n_samples_to_generate = (self.length * 44_100.0).floor() as usize;
        oscillator.generate(n_samples_to_generate)
    }
}

impl Render<Phrase> for Phrase {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        for mut event in self.events.clone() {
            //            println!("{:?}", event);
            let stereo_waveform = event.render(oscillator);
            result.append(stereo_waveform);
        }

        result
    }
}

impl Render<Vec<Phrase>> for Vec<Phrase> {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
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
    use settings::get_test_settings;
    use ratios::Pan;

    fn test_ratios() -> Vec<R> {
        r![
            (1, 1, 0.0, 0.5, 1.0),
            (1, 1, 1.0, 0.5, -1.0)
        ]
    }

    fn test_ratios_change() -> Vec<R> {
        r![
            (3, 2, 0.0, 0.6, -1.0),
            (5, 4, 1.5, 0.5, 0.5)
        ]
    }

    #[test]
    fn test_mutate_event() {
        let result = Event::new(100.0, test_ratios(), 0.1, 1.0)
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Event {
            frequency: 150.0,
            ratios: vec![
                R::atio(3, 2, 0.0, 0.6, Pan::Left),
                R::atio(3, 2, 0.0, 0.0, Pan::Right),
                R::atio(5, 4, 1.5, 0.125, Pan::Left),
                R::atio(5, 4, 1.5, 0.375, Pan::Right),
            ],
            length: 1.2,
            gain: 0.9,
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn test_render_event() {
        let mut result = Event::new(100.0, test_ratios(), 0.001, 1.0)
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 0.0)
            .mut_gain(0.9, 0.0);

        let mut expected = Event {
            frequency: 150.0,
            ratios: vec![
                R::atio(3, 2, 0.0, 0.6, Pan::Left),
                R::atio(3, 2, 0.0, 0.0, Pan::Right),
                R::atio(5, 4, 1.5, 0.125, Pan::Left),
                R::atio(5, 4, 1.5, 0.375, Pan::Right),
            ],
            length: 0.002,
            gain: 0.9,
        };
        let mut oscillator1 = Oscillator::init(test_ratios(), &get_test_settings());
        let mut oscillator2 = Oscillator::init(test_ratios(), &get_test_settings());

        assert_eq!(expected, result);
        assert_eq!(expected.render(&mut oscillator1),
                   result.render(&mut oscillator2));
    }


    #[test]
    fn test_mutate_phrase() {
        let mut phrase = Phrase {
            events: vec![
                Event::new(100.0, test_ratios(), 1.0, 1.0),
                Event::new(50.0, test_ratios(), 2.0, 1.0),
            ],
        };

        let result = phrase
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0);

        let expected = Phrase {
            events: vec![
                Event::new(150.0, test_ratios_change(), 3.0, 0.9),
                Event::new(75.0, test_ratios_change(), 5.0, 0.9),
            ],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_phrase() {
        let mut phrase = Phrase {
            events: vec![
                Event::new(100.0, test_ratios(), 1.0, 1.0),
                Event::new(50.0, test_ratios(), 2.0, 1.0),
            ],
        };

        let mut oscillator1 = Oscillator::init(test_ratios(), &get_test_settings());
        let result = phrase
            .mut_ratios(test_ratios_change())
            .transpose(3.0 / 2.0, 0.0)
            .mut_length(2.0, 1.0)
            .mut_gain(0.9, 0.0)
            .render(&mut oscillator1);

        let mut oscillator2 = Oscillator::init(test_ratios(), &get_test_settings());
        let expected = Phrase {
            events: vec![
                Event::new(150.0, test_ratios_change(), 3.0, 0.9),
                Event::new(75.0, test_ratios_change(), 5.0, 0.9),
            ],
        }.render(&mut oscillator2);
        assert_eq!(result, expected);
    }
}

