use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, Ordering};

// Use atomic pointer for lock-free access in hot paths
// Settings are leaked to get 'static lifetime - this is fine since settings rarely change
static SETTINGS: AtomicPtr<Settings> = AtomicPtr::new(std::ptr::null_mut());

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
    // TUI/Kintaro settings
    pub visual_mode: bool,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    // Mesh settings
    pub mesh_vertices: usize,
    pub mesh_triangles: usize,
    // Instance culling settings
    pub cull_enabled: bool,
    pub cull_max_distance: f32,
    pub cull_behind_threshold: f32,
    // Instance lifetime
    pub max_instance_lifetime: f32,
    // Debug settings
    pub click_detection: bool,
    // Audio render parallelism. The voice-render loop in
    // `audio_engine::render` will parallelize per-voice work when:
    //   - `audio_thread_count > 1`, AND
    //   - voice count is at or above `parallel_voice_threshold`.
    //
    // **Default is `1` (serial)**. The audio engine is reached from the
    // real-time playback path (RenderManager's background renderer
    // thread); rayon dispatch overhead competes with the portaudio
    // callback for cores and causes audible clicks even on heavy
    // compositions where the serial render still runs >100× faster
    // than real time. Bumping this past 1 helps batch benchmarks but
    // hurts real-time playback — the trade is intentional.
    //
    // (Offline `parsed_to_render::render` doesn't use this pool — it
    // parallelizes through rayon's global pool independently, so
    // changing this number doesn't affect `kintaro print` speed.)
    pub audio_thread_count: usize,
    pub parallel_voice_threshold: usize,
}

impl Settings {
    /// Get global settings (fast, returns reference)
    /// Falls back to defaults if not initialized
    pub fn global() -> &'static Settings {
        let ptr = SETTINGS.load(Ordering::Acquire);
        if ptr.is_null() {
            eprintln!("WARNING: Settings accessed before initialization, using defaults");
            // Initialize with defaults - this leaks memory but only once
            let settings = Box::new(default_settings());
            let ptr = Box::into_raw(settings);
            // Try to set it, but if another thread beat us, use theirs
            match SETTINGS.compare_exchange(
                std::ptr::null_mut(),
                ptr,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe { &*ptr },
                Err(other) => {
                    // Another thread set it first, free our allocation and use theirs
                    unsafe { drop(Box::from_raw(ptr)) };
                    unsafe { &*other }
                }
            }
        } else {
            unsafe { &*ptr }
        }
    }

    /// Initialize settings with sample_rate and buffer_size
    /// Loads config files and merges them with provided values
    /// Priority: hardcoded defaults < init params < global config < local config
    pub fn init(sample_rate: f64, buffer_size: usize) {
        let mut settings = default_settings();

        // Apply programmatic defaults
        settings.sample_rate = sample_rate;
        settings.buffer_size = buffer_size;

        // Load and apply config files (they override programmatic defaults)
        if let Some(config) = load_all_configs() {
            config.apply_to(&mut settings);
        }

        Self::set_static(settings);
    }

    /// Initialize with default settings (loads config files)
    /// Can be called multiple times to reload config
    pub fn init_default() {
        let mut settings = default_settings();

        // Load and apply config files
        if let Some(config) = load_all_configs() {
            config.apply_to(&mut settings);
        }

        Self::set_static(settings);
    }

    /// Initialize with test settings
    pub fn init_test() {
        Self::set_static(get_test_settings());
    }

    /// Set settings directly
    pub fn set(&self) {
        Self::set_static(self.clone());
    }

    /// Internal: set the static settings pointer
    /// Leaks the old settings if any (acceptable for rarely-changed config)
    fn set_static(settings: Settings) {
        let new_ptr = Box::into_raw(Box::new(settings));
        let old_ptr = SETTINGS.swap(new_ptr, Ordering::AcqRel);
        // Note: we intentionally leak the old settings to avoid use-after-free
        // This is fine since settings are rarely changed
        let _ = old_ptr; // Suppress unused warning
    }

    /// Check if settings have been initialized
    pub fn is_initialized() -> bool {
        !SETTINGS.load(Ordering::Acquire).is_null()
    }
}

impl Default for Settings {
    fn default() -> Self {
        default_settings()
    }
}

/// Get the path to the global config file (~/.config/weresocool/config.toml)
pub fn config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|d| d.join(".config").join("weresocool").join("config.toml"))
}

/// Ensure the config file exists, creating a default one if not
pub fn ensure_config_exists() -> Option<std::path::PathBuf> {
    let path = config_path()?;
    if !path.exists() {
        // Create parent directory
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Write default config
        let default_config = r#"# WereSoCool Configuration

# Playback
loop_play = false
pad_end = true

# Audio
sample_rate = 48000.0
buffer_size = 12288
channels = 2
interleaved = true

# Rendering
crossfade_period = 4096
lookahead_buffers = 3

# Mic input
mic = false
yin_buffer_size = 2048
probability_threshold = 0.3
gain_threshold_min = 0.0
max_freq = 4500.0
min_freq = 20.0

# Visualization
vis_filter_rate = 0.125
visual_mode = true

# Window (optional)
# window_width = 1280
# window_height = 720
# window_x = 100
# window_y = 100

# Mesh detail (vertices per brush)
# mesh_vertices = 20
# mesh_triangles = 40

# Instance culling (set cull_enabled = false to disable)
# cull_enabled = true
# cull_max_distance = 5.0
# cull_behind_threshold = 0.5

# Instance lifetime (seconds before automatic removal)
# max_instance_lifetime = 30.0

# Audio render parallelism (REAL-TIME PLAYBACK ONLY — does not affect
# `kintaro print` / offline rendering, which parallelizes through rayon's
# global pool independently of this).
#
# audio_thread_count: how many worker threads the per-voice real-time
#   render uses. **Default 1 (serial)**. Going above 1 helps when the
#   composition has many voices and the buffer_size is large, but rayon
#   dispatch overhead competes with the portaudio callback thread for
#   cores and causes audible clicks. Serial already renders >100× faster
#   than real time on tested compositions, so the parallel knob is rarely
#   worth turning up for playback.
# audio_thread_count = 1
#
# parallel_voice_threshold: minimum voice count to fan out to the audio
#   thread pool (only consulted when audio_thread_count > 1). Default 32.
# parallel_voice_threshold = 32
"#;
        let _ = std::fs::write(&path, default_config);
    }
    Some(path)
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
        visual_mode: true,
        window_width: None,
        window_height: None,
        window_x: None,
        window_y: None,
        mesh_vertices: 20,
        mesh_triangles: 40,
        cull_enabled: true,
        cull_max_distance: 5.0,
        cull_behind_threshold: 0.5,
        max_instance_lifetime: 30.0,
        click_detection: false,
        // Serial by default — see comment on the struct field. Bumping
        // this past 1 for real-time playback causes audible clicks.
        audio_thread_count: 1,
        parallel_voice_threshold: 32,
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
    // TUI/Kintaro settings
    visual_mode: Option<bool>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    window_x: Option<i32>,
    window_y: Option<i32>,
    // Mesh settings
    mesh_vertices: Option<usize>,
    mesh_triangles: Option<usize>,
    // Instance culling settings
    cull_enabled: Option<bool>,
    cull_max_distance: Option<f32>,
    cull_behind_threshold: Option<f32>,
    // Instance lifetime
    max_instance_lifetime: Option<f32>,
    // Debug settings
    click_detection: Option<bool>,
    // Audio parallelism
    audio_thread_count: Option<usize>,
    parallel_voice_threshold: Option<usize>,
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
        if let Some(v) = self.visual_mode { settings.visual_mode = v; }
        if self.window_width.is_some() { settings.window_width = self.window_width; }
        if self.window_height.is_some() { settings.window_height = self.window_height; }
        if self.window_x.is_some() { settings.window_x = self.window_x; }
        if self.window_y.is_some() { settings.window_y = self.window_y; }
        if let Some(v) = self.mesh_vertices { settings.mesh_vertices = v; }
        if let Some(v) = self.mesh_triangles { settings.mesh_triangles = v; }
        if let Some(v) = self.cull_enabled { settings.cull_enabled = v; }
        if let Some(v) = self.cull_max_distance { settings.cull_max_distance = v; }
        if let Some(v) = self.cull_behind_threshold { settings.cull_behind_threshold = v; }
        if let Some(v) = self.max_instance_lifetime { settings.max_instance_lifetime = v; }
        if let Some(v) = self.click_detection { settings.click_detection = v; }
        if let Some(v) = self.audio_thread_count { settings.audio_thread_count = v; }
        if let Some(v) = self.parallel_voice_threshold { settings.parallel_voice_threshold = v; }
    }
}

/// Load config file from path
fn load_config_file(path: PathBuf) -> Option<SettingsConfig> {
    if !path.exists() {
        return None;
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(config) => Some(config),
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

/// Helper to apply a loaded config to the merged config
fn apply_config_to_merged(config: &SettingsConfig, merged: &mut SettingsConfig) {
    let mut temp_settings = default_settings();
    config.apply_to(&mut temp_settings);

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
    merged.visual_mode = Some(temp_settings.visual_mode);
    merged.window_width = temp_settings.window_width;
    merged.window_height = temp_settings.window_height;
    merged.window_x = temp_settings.window_x;
    merged.window_y = temp_settings.window_y;
    merged.mesh_vertices = Some(temp_settings.mesh_vertices);
    merged.mesh_triangles = Some(temp_settings.mesh_triangles);
    merged.cull_enabled = Some(temp_settings.cull_enabled);
    merged.cull_max_distance = Some(temp_settings.cull_max_distance);
    merged.cull_behind_threshold = Some(temp_settings.cull_behind_threshold);
    merged.max_instance_lifetime = Some(temp_settings.max_instance_lifetime);
    merged.click_detection = Some(temp_settings.click_detection);
    merged.audio_thread_count = Some(temp_settings.audio_thread_count);
    merged.parallel_voice_threshold = Some(temp_settings.parallel_voice_threshold);
}

/// Load and merge all config files
/// Priority: global (~/.config) < local config
fn load_all_configs() -> Option<SettingsConfig> {
    let mut has_config = false;
    let mut merged = SettingsConfig::default();

    // Load global config from ~/.config/weresocool/config.toml
    if let Some(home) = dirs::home_dir() {
        let config_path = home.join(".config").join("weresocool").join("config.toml");
        if let Some(config) = load_config_file(config_path) {
            apply_config_to_merged(&config, &mut merged);
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
    if source.visual_mode.is_some() { dest.visual_mode = source.visual_mode; }
    if source.window_width.is_some() { dest.window_width = source.window_width; }
    if source.window_height.is_some() { dest.window_height = source.window_height; }
    if source.window_x.is_some() { dest.window_x = source.window_x; }
    if source.window_y.is_some() { dest.window_y = source.window_y; }
    if source.mesh_vertices.is_some() { dest.mesh_vertices = source.mesh_vertices; }
    if source.mesh_triangles.is_some() { dest.mesh_triangles = source.mesh_triangles; }
    if source.cull_enabled.is_some() { dest.cull_enabled = source.cull_enabled; }
    if source.cull_max_distance.is_some() { dest.cull_max_distance = source.cull_max_distance; }
    if source.cull_behind_threshold.is_some() { dest.cull_behind_threshold = source.cull_behind_threshold; }
    if source.max_instance_lifetime.is_some() { dest.max_instance_lifetime = source.max_instance_lifetime; }
    if source.click_detection.is_some() { dest.click_detection = source.click_detection; }
    if source.audio_thread_count.is_some() { dest.audio_thread_count = source.audio_thread_count; }
    if source.parallel_voice_threshold.is_some() { dest.parallel_voice_threshold = source.parallel_voice_threshold; }
}
