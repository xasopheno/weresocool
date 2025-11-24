// Synthesis modules provided by weresocool_synth

// Bridge layer - converts WereSoCool AST to synthesis operations
pub mod renderable;

// Re-export types from weresocool_synth
pub use weresocool_synth::{
    Basis, Oscillator, StereoWaveform, Normalize, Voice, SynthOp, Offset,
};

// Re-export instrument-specific types (bridge layer)
pub use self::{
    renderable::render_voice::{renderables_to_render_voices, RenderVoice},
    renderable::RenderOp,
};
