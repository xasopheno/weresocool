use crate::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize)]
pub enum AdsrState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AdsrEnvelope {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
    pub time: f64,
    pub state: AdsrState,
}

impl Eq for AdsrEnvelope {}

impl Ord for AdsrEnvelope {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Hash for AdsrEnvelope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.attack.to_bits().hash(state);
        self.decay.to_bits().hash(state);
        self.sustain.to_bits().hash(state);
        self.release.to_bits().hash(state);
        self.time.to_bits().hash(state);
        // Assuming AdsrState also implements Hash
        self.state.hash(state);
    }
}

impl AdsrEnvelope {
    pub fn new(attack: f64, sustain: f64, decay: f64, release: f64) -> Self {
        AdsrEnvelope {
            attack,
            decay,
            sustain,
            release,
            time: 0.0,
            state: AdsrState::Off,
        }
    }

    pub fn process(&mut self, sample_rate: f64) -> f64 {
        let gain = match self.state {
            AdsrState::Attack => self.time / self.attack,
            AdsrState::Decay => {
                self.sustain + (1.0 - self.sustain) * (1.0 - self.time / self.decay)
            }
            AdsrState::Sustain => self.sustain,
            AdsrState::Release => self.sustain * (1.0 - self.time / self.release),
            AdsrState::Off => 0.0,
        };
        self.time += 1.0 / sample_rate;
        gain
    }

    pub fn change_state(&mut self, new_state: AdsrState) {
        self.state = new_state;
        self.time = 0.0;
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub struct AdsrFilter {
    pub envelope: AdsrEnvelope,
    pub biquad_filter: BiquadFilter,
}

impl AdsrFilter {
    pub fn new(
        cutoff_frequency: f64,
        quality_factor: f64,
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> Self {
        AdsrFilter {
            envelope: AdsrEnvelope::new(attack, decay, sustain, release),
            biquad_filter: lowpass_filter(cutoff_frequency, quality_factor),
        }
    }

    pub fn process(&mut self, input_sample: f64, sample_rate: f64) -> f64 {
        let gain = self.envelope.process(sample_rate);
        gain * self.biquad_filter.process(input_sample)
    }

    pub fn reset(&mut self) {
        self.envelope.change_state(AdsrState::Off);
        self.biquad_filter.input_history = [0.0; 2];
        self.biquad_filter.output_history = [0.0; 2];
    }
}
