extern crate colored;
extern crate pbr;
extern crate portaudio;
extern crate serde;
extern crate serde_json;
extern crate socool_parser;
#[macro_use]
extern crate serde_derive;
extern crate difference;
extern crate indexmap;
extern crate num_rational;
extern crate rand;
extern crate rayon;
extern crate term;

#[macro_use]
pub mod macros;
pub mod analyze;
pub mod compositions;
pub mod examples;
pub mod generation;
pub mod instrument;
pub mod operations;
pub mod portaudio_setup;
pub mod render;
pub mod ring_buffer;
pub mod settings;
pub mod testing;
pub mod ui;
pub mod write;
