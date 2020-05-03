#![warn(
    clippy::nursery,
    //clippy::restriction,
    //clippy::pedantic,
    //clippy::cargo,
)]
pub mod analyze;
pub mod examples;
pub mod generation;
pub mod helpers;
pub mod instrument;
pub mod interpretable;
pub mod manager;
pub mod portaudio;
pub mod renderable;
pub mod ring_buffer;
pub mod settings;
pub mod testing;
pub mod ui;
pub mod write;
