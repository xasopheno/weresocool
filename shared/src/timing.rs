use std::sync::OnceLock;

/// Check if timing output is enabled via WSC_TIMING environment variable
#[inline]
pub fn timing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WSC_TIMING").is_ok())
}

/// Print timing info only if WSC_TIMING environment variable is set
#[macro_export]
macro_rules! timing_print {
    ($($arg:tt)*) => {
        if $crate::timing::timing_enabled() {
            eprintln!($($arg)*);
        }
    };
}
