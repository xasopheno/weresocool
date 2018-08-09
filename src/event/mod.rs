use instrument::{oscillator::Oscillator, stereo_waveform::StereoWaveform};
use ratios::{Pan, R};

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

impl Render<Vec<Event>> for Vec<Event> {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut events = self.clone();
        let r = r![(0, 1, 0.0, 0.0, 0.0)];
        events.push(Event::new(0.0, r, 3.0, 0.0));
        for mut event in events {
            let stereo_waveform = event.render(oscillator);
            result.append(stereo_waveform);
        }

        result
    }
}

#[cfg(test)]
mod test;
