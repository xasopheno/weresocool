use once_cell::sync::OnceCell;

static SETTINGS: OnceCell<Settings> = if cfg!(test) {
    OnceCell::with_value(get_test_settings())
} else {
    // OnceCell::with_value(default_settings())
    OnceCell::new()
};

impl Settings {
    pub fn global() -> &'static Settings {
        SETTINGS.get().expect("Oh no! Settings are not initialized")
    }

    pub fn init(sample_rate: f64, buffer_size: usize) -> Result<(), std::io::Error> {
        SETTINGS
            .set(Settings {
                sample_rate,
                buffer_size,
                ..get_test_settings()
            })
            .expect("Settings already initialized");
        Ok(())
    }

    pub fn init_default() -> Result<(), std::io::Error> {
        SETTINGS
            .set(default_settings())
            .expect("Settings already initialized");
        Ok(())
    }

    pub fn init_test() -> Result<(), std::io::Error> {
        SETTINGS
            .set(get_test_settings())
            .expect("Settings already initialized");
        Ok(())
    }
}

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
}

pub const fn default_settings() -> Settings {
    Settings {
        loop_play: false,
        pad_end: true,
        mic: false,
        sample_rate: 48_000.0,
        yin_buffer_size: 2048,
        buffer_size: 1024 * 8,
        // buffer_size: 512,
        probability_threshold: 0.3,
        gain_threshold_min: 0.0,
        channels: 2,
        interleaved: true,
        max_freq: 2_500.0,
        min_freq: 20.0,
    }
}

pub const fn get_test_settings() -> Settings {
    Settings {
        sample_rate: 44_100.0,
        ..default_settings()
    }
}
