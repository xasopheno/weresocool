use std::sync::atomic::AtomicUsize;

struct Shared {
    frequency: f32,
    probability: f32,
    gain: f32,
}

struct SharedUpdater {
    frequency: Option<f32>,
    probability: Option<f32>,
    gain: Option<f32>,
}

impl Shared {
    pub fn new() -> Shared {
        Shared {
            frequency: 0.0,
            probability: 0.0,
            gain: 0.0,
        }
    }

    pub fn update(&mut self, updater: SharedUpdater) {
        match updater.frequency {
          None => None,
          Some(frequency) => self.frequency = frequency,
        };
        match updater.probability {
            None => None,
            Some(probability) => self.probability = probability,
        };
        match updater.gain {
            None => None,
            Some(gain) => self.gain = gain,
        };
    }
}