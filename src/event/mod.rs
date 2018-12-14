use instrument::{
    oscillator::{OscType, Oscillator},
    stereo_waveform::StereoWaveform,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Event {
    pub sounds: Vec<Sound>,
    pub length: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sound {
    pub frequency: f64,
    pub gain: f64,
    pub pan: f64,
    pub osc_type: OscType,
}

impl Sound {
    pub fn init() -> Sound {
        Sound {
            frequency: 0.0,
            gain: 0.0,
            pan: 0.0,
            osc_type: OscType::Sine,
        }
    }
}

impl Event {
    pub fn init(frequency: f64, gain: f64, pan: f64, length: f64) -> Event {
        Event {
            sounds: {
                vec![Sound {
                    frequency,
                    gain,
                    pan,
                    osc_type: OscType::Sine,
                }]
            },
            length,
        }
    }
}

pub trait Render<T> {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform;
}

impl Render<Event> for Event {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        oscillator.update(self.sounds.clone());
        let n_samples_to_generate = self.length * 44_100.0;
        oscillator.generate(n_samples_to_generate)
    }
}

impl Render<Vec<Event>> for Vec<Event> {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut events = self.clone();
        events.push(Event::init(0.0, 0.0, 0.0, 1.0));
        for mut event in events {
            let stereo_waveform = event.render(oscillator);
            result.append(stereo_waveform);
        }

        result
    }
}

#[cfg(test)]
mod test;
