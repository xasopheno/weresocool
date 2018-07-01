use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc};

struct State {
    frequency: AtomicU32,
    probability: AtomicU32,
    gain: AtomicU32,
}

struct StateAPI {
    frequency: Option<f32>,
    probability: Option<f32>,
    gain: Option<f32>,
}

impl State {
    pub fn new() -> Arc<State> {
        Arc::new(State {
            frequency: AtomicU32::new(0),
            probability: AtomicU32::new(0),
            gain: AtomicU32::new(0),
        })
    }

    pub fn update(&mut self, updater: StateAPI) {
        match updater.frequency {
          Some(frequency) => self.frequency.store(frequency.to_bits(), Ordering::Relaxed),
          None => {},
        }
        match updater.probability {
            Some(probability) => self.probability.store(probability.to_bits(), Ordering::Relaxed),
            None => {}
        }
        match updater.gain {
            Some(gain) => self.gain.store(gain.to_bits(), Ordering::Relaxed),
            None => {},
        }
    }

    pub fn get_state(&mut self) -> StateAPI {
        StateAPI {
            frequency: Some(f32::from_bits(self.frequency.load(Ordering::Relaxed))),
            probability: Some(f32::from_bits(self.frequency.load(Ordering::Relaxed))),
            gain: Some(f32::from_bits(self.frequency.load(Ordering::Relaxed))),
        }
    }
}
