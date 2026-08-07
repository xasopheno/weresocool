// weresocool_synth - Generic audio synthesis engine
// Standalone and reusable, independent of WereSoCool language types

mod asr;
pub mod distortion;
mod frequency;
mod gain;
mod loudness;
pub mod oscillator;
pub mod presets;
mod sample;
pub mod tables;
pub mod stereo_waveform;
pub mod voice;

#[cfg(test)]
mod asr_test;

// Re-export key types
pub use self::{
    distortion::{DistortionDef, process_distortions},
    oscillator::{Basis, Oscillator},
    stereo_waveform::{Normalize, StereoWaveform},
    voice::Voice,
};

// Re-export from dependencies
pub use weresocool_ast::{OscType, ASR};
pub use weresocool_filter::BiquadFilterDef;

/// Generic synthesis operation interface
///
/// This trait defines the interface that any synthesis operation must implement
/// to be rendered by the synthesis engine. It provides all the parameters needed
/// for audio generation without coupling to any specific language or composition system.
///
/// All methods should be zero-cost abstractions (inline to direct field access).
pub trait SynthOp: Send + Sync {
    // Core audio parameters

    /// Frequency in Hz
    fn frequency(&self) -> f64;

    /// Left channel gain (0.0 to 2.0+)
    fn gain_left(&self) -> f64;

    /// Right channel gain (0.0 to 2.0+)
    fn gain_right(&self) -> f64;

    /// Pan position (-1.0 to 1.0)
    fn pan(&self) -> f64;

    /// Duration in samples
    fn duration_samples(&self) -> usize;

    // Synthesis parameters

    /// Oscillator type (Sine, Saw, Square, etc.)
    fn oscillator_type(&self) -> &OscType;

    /// Biquad filter definitions to apply
    fn filters(&self) -> &[BiquadFilterDef];

    /// Attack envelope duration in samples
    fn envelope_attack(&self) -> f64;

    /// Decay envelope duration in samples
    fn envelope_decay(&self) -> f64;

    /// ASR (Attack/Sustain/Release) envelope type
    fn asr_type(&self) -> ASR;

    /// Portamento (pitch slide) amount (0-1024+)
    fn portamento(&self) -> usize;

    /// Reverb amount (0.0 to 1.0), None = no reverb
    fn reverb(&self) -> Option<f64>;

    /// Initial oscillator phase (radians) to seed at voice birth.
    /// `None` = legacy behavior (phase integrates from 0).
    fn initial_phase(&self) -> Option<f64> {
        None
    }

    /// Distortion effects to apply (wavefolder, etc.) - stackable
    fn distortions(&self) -> &[DistortionDef];

    // State tracking

    /// Current sample index within the operation
    fn sample_index(&self) -> usize;

    /// Total samples in the operation
    fn total_samples(&self) -> usize;

    /// Whether the next operation's left channel is silent (for envelope)
    fn next_left_silent(&self) -> bool;

    /// Whether the next operation's right channel is silent (for envelope)
    fn next_right_silent(&self) -> bool;

    /// Whether this operation ends the voice (reset state)
    fn next_out(&self) -> bool;
}

/// Offset modulation for voice rendering
/// Used to apply external modulation sources (e.g., LFO, follower, randomness)
#[derive(Debug, Clone, Copy)]
pub struct Offset {
    pub freq: f64,
    pub gain: f64,
}

impl Offset {
    pub const fn default() -> Self {
        Self {
            freq: 1.0,
            gain: 1.0,
        }
    }
}
