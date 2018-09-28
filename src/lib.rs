#![feature(extern_prelude)]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate portaudio;
#[macro_use]
pub mod macros;
pub mod analyze;
pub mod compositions;
pub mod event;
pub mod instrument;
pub mod operations;
pub mod portaudio_setup;
pub mod ring_buffer;
pub mod settings;
pub mod write;
