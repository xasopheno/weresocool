use ratios::{R, StereoRatios};

#[derive(Debug)]
pub struct Event {
    pub frequency: f32,
    pub ratios: StereoRatios,
    pub length: f32,
    pub gain: f32,
}

#[derive(Debug)]
pub struct Phrase {
    pub events: Vec<Event>,
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

pub trait Mutate {
    fn transpose(&mut self, mul: f32, add: f32) -> &mut Event;
    fn mut_ratios(&mut self, ratios: StereoRatios);
    fn mut_length(&mut self, mul: f32, add: f32);
    fn mut_gain(&mut self, mul: f32, add: f32);
}

impl Mutate for Event {
    fn transpose(&mut self, mul: f32, add: f32) -> &mut Event {
        self.frequency = self.frequency * mul + add;
        self
    }

    fn mut_ratios(&mut self, ratios: StereoRatios) {
        self.ratios = ratios;
    }

    fn mut_length(&mut self, mul: f32, add: f32) {
        self.length = self.length * mul + add;
    }

    fn mut_gain(&mut self, mul: f32, add: f32) {
        self.gain = self.gain * mul + add;
    }
}

//impl Mutate for Phrase {
//    fn transpose(&mut self, mul: f32, add: f32) {
//        for event in self.events.iter_mut() {
//            event.transpose(mul, add);
//        }
//    }
//
//    fn mut_ratios(&mut self, ratios: StereoRatios) {
//        for event in self.events.iter_mut() {
//            event.mut_ratios(ratios.clone());
//        }
//    }
//
//    fn mut_length(&mut self, mul: f32, add: f32) {
//        for event in self.events.iter_mut() {
//            event.mut_length(mul, add);
//        }
//    }
//
//    fn mut_gain(&mut self, mul: f32, add: f32) {
//        for event in self.events.iter_mut() {
//            event.mut_gain(mul, add);
//        }
//    }
//}