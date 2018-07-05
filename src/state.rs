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
            frequency: 0.0,
            probability: 0.0,
            gain: 0.0,
        }
    }

    pub fn update(&mut self, update: StateAPI) {
        self.frequency = update.frequency;
        self.probability = update.probability;
        self.gain = update.gain;
    }

    pub fn get_state(&mut self) -> StateAPI {
        StateAPI {
            frequency: self.frequency,
            probability: self.frequency,
            gain: self.frequency,
        }
    }
}
