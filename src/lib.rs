#![feature(extern_prelude)]
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
extern crate pbr;
extern crate socool_parser;
extern crate portaudio;
extern crate colored;

#[macro_use]
pub mod macros;
pub mod generation;
pub mod analyze;
pub mod compositions;
pub mod event;
pub mod instrument;
pub mod operations;
pub mod portaudio_setup;
pub mod ring_buffer;
pub mod settings;
pub mod write;


