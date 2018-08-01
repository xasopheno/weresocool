#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub sample_rate: f32,
    pub yin_buffer_size: usize,
    pub buffer_size: usize,
    pub probability_threshold: f32,
    pub gain_threshold_min: f32,
    pub channels: i32,
    pub interleaved: bool,
    pub max_freq: f32,
    pub min_freq: f32,
    pub gain_multiplier: f32,
}

pub fn get_default_app_settings() -> Settings {
    Settings {
        sample_rate: 44_100.0,
        yin_buffer_size: 2048,
        buffer_size: 2048,
        probability_threshold: 0.3,
        gain_threshold_min: 0.000,
        channels: 2,
        interleaved: true,
        max_freq: 2_500.0,
        min_freq: 20.0,
        gain_multiplier: 10.0,
    }
}

pub fn get_test_settings() -> Settings {
    Settings {
        buffer_size: 10,
        ..get_default_app_settings()
    }
}
