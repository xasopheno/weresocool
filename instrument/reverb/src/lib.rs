//! A stereo plate reverberator developed by Lance Putnam, ported to Rust by MindBuffer.

pub use reverb::Reverb;

mod delay_line;
mod reverb;

#[cfg(feature="dsp-chain")]
mod dsp_node;
