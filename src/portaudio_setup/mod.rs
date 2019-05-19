pub mod duplex;
pub mod output;

pub use crate::portaudio_setup::{
    duplex::setup_portaudio_duplex,
    output::setup_portaudio_output
};
