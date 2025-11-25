#[cfg(not(test))]
use once_cell::sync::OnceCell;

use serde::Deserialize;
use std::path::PathBuf;

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
    /// Loads config files and merges them with provided values
    /// Priority: hardcoded defaults < init params < global config < local config
    #[cfg(not(test))]
    pub fn init(sample_rate: f64, buffer_size: usize) {
        let mut settings = default_settings();

        // Apply programmatic defaults
        settings.sample_rate = sample_rate;
        settings.buffer_size = buffer_size;

        // Load and apply config files (they override programmatic defaults)
        if let Some(config) = load_all_configs() {
            config.apply_to(&mut settings);
        }

        _ = SETTINGS.set(settings);
    }

    /// Initialize settings in test mode (skips config file loading for test isolation)
    #[cfg(test)]
    pub fn init(sample_rate: f64, buffer_size: usize) {
        let mut settings = default_settings();
        settings.sample_rate = sample_rate;
        settings.buffer_size = buffer_size;
        *SETTINGS.write().expect("Failed to write Settings lock") = settings;
    }

    /// Initialize with default settings (loads config files)
    #[cfg(not(test))]
    pub fn init_default() {
        let mut settings = default_settings();

        // Load and apply config files
        if let Some(config) = load_all_configs() {
            config.apply_to(&mut settings);
        }

        _ = SETTINGS.set(settings);
    }

    /// Initialize with default settings in test mode (skips config loading)
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

/// Configuration structure for loading from TOML files
/// All fields are optional to support partial configs
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct SettingsConfig {
    pad_end: Option<bool>,
    loop_play: Option<bool>,
    mic: Option<bool>,
    sample_rate: Option<f64>,
    yin_buffer_size: Option<usize>,
    buffer_size: Option<usize>,
    probability_threshold: Option<f32>,
    gain_threshold_min: Option<f32>,
    channels: Option<i32>,
    interleaved: Option<bool>,
    max_freq: Option<f64>,
    min_freq: Option<f64>,
    crossfade_period: Option<usize>,
    lookahead_buffers: Option<usize>,
    vis_filter_rate: Option<f32>,
}

impl SettingsConfig {
    /// Merge this config into a Settings struct
    fn apply_to(&self, settings: &mut Settings) {
        if let Some(v) = self.pad_end { settings.pad_end = v; }
        if let Some(v) = self.loop_play { settings.loop_play = v; }
        if let Some(v) = self.mic { settings.mic = v; }
        if let Some(v) = self.sample_rate { settings.sample_rate = v; }
        if let Some(v) = self.yin_buffer_size { settings.yin_buffer_size = v; }
        if let Some(v) = self.buffer_size { settings.buffer_size = v; }
        if let Some(v) = self.probability_threshold { settings.probability_threshold = v; }
        if let Some(v) = self.gain_threshold_min { settings.gain_threshold_min = v; }
        if let Some(v) = self.channels { settings.channels = v; }
        if let Some(v) = self.interleaved { settings.interleaved = v; }
        if let Some(v) = self.max_freq { settings.max_freq = v; }
        if let Some(v) = self.min_freq { settings.min_freq = v; }
        if let Some(v) = self.crossfade_period { settings.crossfade_period = v; }
        if let Some(v) = self.lookahead_buffers { settings.lookahead_buffers = v; }
        if let Some(v) = self.vis_filter_rate { settings.vis_filter_rate = v; }
    }
}

/// Load config file from path
fn load_config_file(path: PathBuf) -> Option<SettingsConfig> {
    if !path.exists() {
        return None;
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(config) => {
                eprintln!("Loaded config from: {}", path.display());
                Some(config)
            }
            Err(e) => {
                eprintln!("WARNING: Failed to parse config file {}: {}", path.display(), e);
                None
            }
        },
        Err(e) => {
            eprintln!("WARNING: Failed to read config file {}: {}", path.display(), e);
            None
        }
    }
}

/// Load and merge all config files
/// Priority: global config < local config (local overrides global)
fn load_all_configs() -> Option<SettingsConfig> {
    let mut has_config = false;
    let mut merged = SettingsConfig::default();

    // Load global config: ~/.config/weresocool/config.toml
    if let Some(config_dir) = dirs::config_dir() {
        let global_config_path = config_dir.join("weresocool").join("config.toml");
        if let Some(global_config) = load_config_file(global_config_path) {
            // Apply global config to a temporary Settings, then extract back
            let mut temp_settings = default_settings();
            global_config.apply_to(&mut temp_settings);

            // Store the values in merged
            merged.pad_end = Some(temp_settings.pad_end);
            merged.loop_play = Some(temp_settings.loop_play);
            merged.mic = Some(temp_settings.mic);
            merged.sample_rate = Some(temp_settings.sample_rate);
            merged.yin_buffer_size = Some(temp_settings.yin_buffer_size);
            merged.buffer_size = Some(temp_settings.buffer_size);
            merged.probability_threshold = Some(temp_settings.probability_threshold);
            merged.gain_threshold_min = Some(temp_settings.gain_threshold_min);
            merged.channels = Some(temp_settings.channels);
            merged.interleaved = Some(temp_settings.interleaved);
            merged.max_freq = Some(temp_settings.max_freq);
            merged.min_freq = Some(temp_settings.min_freq);
            merged.crossfade_period = Some(temp_settings.crossfade_period);
            merged.lookahead_buffers = Some(temp_settings.lookahead_buffers);
            merged.vis_filter_rate = Some(temp_settings.vis_filter_rate);
            has_config = true;
        }
    }

    // Load local config: ./weresocool.toml or ./.weresocool.toml
    if let Ok(current_dir) = std::env::current_dir() {
        // Try weresocool.toml first
        let local_config_path = current_dir.join("weresocool.toml");
        if let Some(local_config) = load_config_file(local_config_path) {
            // Local config overrides global - apply it on top
            merge_configs(&mut merged, local_config);
            has_config = true;
        } else {
            // Try .weresocool.toml as fallback
            let hidden_config_path = current_dir.join(".weresocool.toml");
            if let Some(local_config) = load_config_file(hidden_config_path) {
                merge_configs(&mut merged, local_config);
                has_config = true;
            }
        }
    }

    if has_config {
        Some(merged)
    } else {
        None
    }
}

/// Merge source config into destination (source overrides destination)
fn merge_configs(dest: &mut SettingsConfig, source: SettingsConfig) {
    if source.pad_end.is_some() { dest.pad_end = source.pad_end; }
    if source.loop_play.is_some() { dest.loop_play = source.loop_play; }
    if source.mic.is_some() { dest.mic = source.mic; }
    if source.sample_rate.is_some() { dest.sample_rate = source.sample_rate; }
    if source.yin_buffer_size.is_some() { dest.yin_buffer_size = source.yin_buffer_size; }
    if source.buffer_size.is_some() { dest.buffer_size = source.buffer_size; }
    if source.probability_threshold.is_some() { dest.probability_threshold = source.probability_threshold; }
    if source.gain_threshold_min.is_some() { dest.gain_threshold_min = source.gain_threshold_min; }
    if source.channels.is_some() { dest.channels = source.channels; }
    if source.interleaved.is_some() { dest.interleaved = source.interleaved; }
    if source.max_freq.is_some() { dest.max_freq = source.max_freq; }
    if source.min_freq.is_some() { dest.min_freq = source.min_freq; }
    if source.crossfade_period.is_some() { dest.crossfade_period = source.crossfade_period; }
    if source.lookahead_buffers.is_some() { dest.lookahead_buffers = source.lookahead_buffers; }
    if source.vis_filter_rate.is_some() { dest.vis_filter_rate = source.vis_filter_rate; }
}
