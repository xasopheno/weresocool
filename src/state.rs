use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct State {
    frequency: f32,
    probability: f32,
    gain: f32,
}

#[derive(Debug)]
pub struct StateAPI {
    pub frequency: f32,
    pub probability: f32,
    pub gain: f32,
}

impl State {
    pub fn new() -> State {
        State {
            frequency: AtomicU32::new(0),
            probability: AtomicU32::new(0),
            gain: AtomicU32::new(0),
        }
    }

    pub fn update(&mut self, update: StateAPI) {
        self.frequency
            .store(update.frequency.to_bits(), Ordering::Relaxed);
        self.probability
            .store(update.probability.to_bits(), Ordering::Relaxed);
        self.gain.store(update.gain.to_bits(), Ordering::Relaxed);
    }

    pub fn get_state(&mut self) -> StateAPI {
        StateAPI {
            frequency: f32::from_bits(self.frequency.load(Ordering::Relaxed)),
            probability: f32::from_bits(self.frequency.load(Ordering::Relaxed)),
            gain: f32::from_bits(self.frequency.load(Ordering::Relaxed)),
        }
    }
}
