#[cfg(not(test))]
use once_cell::sync::OnceCell;

#[cfg(not(test))]
static SETTINGS: OnceCell<Settings> = OnceCell::new();

#[cfg(test)]
static SETTINGS: std::sync::RwLock<Settings> = std::sync::RwLock::new(get_test_settings());

/// Global settings for WereSoCool audio rendering
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub pad_end: bool,
    pub loop_play: bool,
    pub mic: bool,
    pub sample_rate: f64,
    pub yin_buffer_size: usize,
    pub buffer_size: usize,
    pub probability_threshold: f32,
    pub gain_threshold_min: f32,
    pub channels: i32,
    pub interleaved: bool,
    pub max_freq: f64,
    pub min_freq: f64,
    pub crossfade_period: usize,
    pub lookahead_buffers: usize,
    pub vis_filter_rate: f32,
}

impl Settings {
    /// Get global settings
    #[cfg(not(test))]
    pub fn global() -> &'static Settings {
        SETTINGS.get_or_init(|| {
            eprintln!("WARNING: Settings accessed before initialization, using defaults");
            default_settings()
        })
    }

    /// Get global settings (test version returns a guard that derefs to Settings)
    #[cfg(test)]
    pub fn global() -> std::sync::RwLockReadGuard<'static, Settings> {
        SETTINGS.read().expect("Failed to read Settings lock")
    }

    /// Initialize settings with sample_rate and buffer_size
    #[cfg(not(test))]
    pub fn init(sample_rate: f64, buffer_size: usize) {
        let mut settings = default_settings();
        settings.sample_rate = sample_rate;
        settings.buffer_size = buffer_size;
        _ = SETTINGS.set(settings);
    }

    #[cfg(test)]
    pub fn init(sample_rate: f64, buffer_size: usize) {
        let mut settings = default_settings();
        settings.sample_rate = sample_rate;
        settings.buffer_size = buffer_size;
        *SETTINGS.write().expect("Failed to write Settings lock") = settings;
    }

    /// Initialize with default settings
    #[cfg(not(test))]
    pub fn init_default() {
        _ = SETTINGS.set(default_settings());
    }

    #[cfg(test)]
    pub fn init_default() {
        *SETTINGS.write().expect("Failed to write Settings lock") = default_settings();
    }

    /// Initialize with test settings
    #[cfg(not(test))]
    pub fn init_test() {
        _ = SETTINGS.set(get_test_settings());
    }

    #[cfg(test)]
    pub fn init_test() {
        *SETTINGS.write().expect("Failed to write Settings lock") = get_test_settings();
    }

    /// Set settings (production only - tests should use set_for_test)
    #[cfg(not(test))]
    pub fn set(&self) {
        _ = SETTINGS.set(self.clone());
    }

    /// Update settings in test mode
    /// This allows tests to reconfigure Settings between assertions
    #[cfg(test)]
    pub fn set_for_test(settings: Settings) {
        *SETTINGS.write().expect("Failed to write Settings lock") = settings;
    }
}

impl Default for Settings {
    fn default() -> Self {
        default_settings()
    }
}

/// Get default settings
pub const fn default_settings() -> Settings {
    Settings {
        loop_play: false,
        pad_end: true,
        mic: false,
        sample_rate: 48_000.0,
        yin_buffer_size: 2048,
        buffer_size: 1024 * 12,
        crossfade_period: 1024 * 4,
        probability_threshold: 0.3,
        gain_threshold_min: 0.0,
        channels: 2,
        interleaved: true,
        max_freq: 4_500.0,
        min_freq: 20.0,
        lookahead_buffers: 3,
        vis_filter_rate: 0.125,
    }
}

/// Get test settings
pub const fn get_test_settings() -> Settings {
    Settings {
        sample_rate: 44_100.0,
        ..default_settings()
    }
}
