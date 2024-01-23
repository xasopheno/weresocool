#![warn(
    clippy::nursery,
    clippy::suspicious,
    clippy::correctness,
    clippy::complexity,
    // clippy::restriction,
    // clippy::pedantic,
    // clippy::cargo,
)]

pub use crate::manager::RenderManager;
pub use weresocool_ast as ast;
pub use weresocool_core::generation::*;
pub use weresocool_core::interpretable::*;
pub use weresocool_core::*;
pub use weresocool_error as error;
pub use weresocool_instrument as instrument;
pub use weresocool_instrument::renderable::*;
pub use weresocool_instrument::StereoWaveform;
pub use weresocool_parser as parser;
pub use weresocool_shared as shared;
pub use weresocool_shared::Settings;
pub mod testing;
