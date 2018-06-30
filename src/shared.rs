use std::sync::atomic::{AtomicUsize, Ordering, Arc};

struct Shared {
    frequency: Arc,
    probability: Arc,
    gain: Arc,
}

struct SharedAPI {
    frequency: Option<f32>,
    probability: Option<f32>,
    gain: Option<f32>,
}

impl Shared {
    pub fn new() -> Shared {
        Shared {
            frequency: Arc::new(AtomicUsize::new(0)),
            probability: AtomicUsize::new(0),
            gain: AtomicUsize::new(0),
        }
    }

    pub fn update(&mut self, updater: SharedAPI) {
        match updater.frequency {
          None => None,
          Some(frequency) => self.frequency = frequency.to_bits(),
        };
        match updater.probability {
            None => None,
            Some(probability) => self.probability = probability.to_bits(),
        };
        match updater.gain {
            None => None,
            Some(gain) => self.gain = gain.to_bits(),
        };
    }

    pub fn get_current_values(&mut self) -> SharedAPI {
        SharedAPI {
            frequency: frequency.from_bits(),
            probability: frequency.from_bits(),
            gain: frequency.from_bits(),
        }
    }
}
