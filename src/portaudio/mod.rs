#[cfg(feature = "app")]
pub mod duplex;
// pub mod output;
#[cfg(feature = "app")]
pub mod real_time;
#[cfg(feature = "app")]
pub mod real_time_buffer_manager;
#[cfg(feature = "app")]
pub mod real_time_render_manager;

#[cfg(feature = "app")]
pub use self::duplex::duplex_setup;
// pub use self::output::output_setup;
#[cfg(feature = "app")]
pub use self::real_time::real_time;
#[cfg(feature = "app")]
pub use self::real_time_buffer_manager::real_time_buffer_manager;
#[cfg(feature = "app")]
pub use self::real_time_render_manager::real_time_render_manager;
