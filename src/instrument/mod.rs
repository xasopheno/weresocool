mod asr;
mod loudness;
pub mod oscillator;
mod sample;
pub mod stereo_waveform;
#[cfg(test)]
mod test;
pub mod voice;

pub use crate::instrument::oscillator::{Basis, Oscillator};
pub use crate::instrument::stereo_waveform::{Normalize, StereoWaveform};
