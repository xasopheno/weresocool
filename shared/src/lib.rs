pub mod helpers;
mod settings;

pub use helpers::{f32_string_to_rational, lossy_rational_mul, r_to_f64};
pub use settings::{default_settings, get_settings, get_test_settings, Settings};
