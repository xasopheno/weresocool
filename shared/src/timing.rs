/// Check if timing output is enabled via WSC_TIMING environment variable.
/// Not available on wasm32 (no env / no std::time).
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn timing_enabled() -> bool {
    use std::sync::OnceLock;
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WSC_TIMING").is_ok())
}

/// Start a timing measurement. Returns an `Instant` on native, a unit `()` on wasm32
/// (where `std::time::Instant` panics with "time not implemented on this platform").
/// Pair with `timing_print!` — on wasm32 the print is a no-op so `.elapsed()` is never
/// evaluated.
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! timing_now {
    () => { std::time::Instant::now() };
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! timing_now {
    () => { () };
}

/// Print timing info only if WSC_TIMING environment variable is set.
/// On wasm32 this expands to nothing — args are not evaluated, so calls like
/// `timing_print!("...", start.elapsed())` are safe even when `start` is unit.
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! timing_print {
    ($($arg:tt)*) => {
        if $crate::timing::timing_enabled() {
            eprintln!($($arg)*);
        }
    };
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! timing_print {
    ($($arg:tt)*) => {};
}
