pub mod duplex;
// pub mod real_time;
pub mod real_time_buffer_manager;
pub mod real_time_render_manager;
pub mod real_time_render_manager_mic;
pub mod server_render_manager;

pub use self::duplex::duplex_setup;
// pub use self::real_time::real_time;
pub use self::real_time_buffer_manager::real_time_buffer_manager;
pub use self::real_time_render_manager::real_time_render_manager;
pub use self::real_time_render_manager_mic::real_time_render_manager_mic;
pub use self::server_render_manager::server_render_manager;
