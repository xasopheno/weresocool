#[derive(Clone)]
pub struct Settings {
    pub sample_rate: f32,
    pub yin_buffer_size: usize,
    pub buffer_size: usize,
    pub probability_threshold: f32,
    pub gain_threshold: f32,
    pub gain_threshold_min: f32,
    pub channels: i32,
    pub interleaved: bool,
    pub maximum_frequency: f32,
    pub minimum_frequency: f32,
}

pub fn get_default_app_settings() -> Settings {
    Settings {
        sample_rate: 44_100.0,
        yin_buffer_size: 2048,
        buffer_size: 512,
        probability_threshold: 0.2,
        gain_threshold: 0.0,
        gain_threshold_min: 0.008,
        channels: 2,
        interleaved: true,
        maximum_frequency: 2_500.0,
        minimum_frequency: 0.0,
    }
}

pub fn get_test_settings() -> Settings {
    Settings {
        buffer_size: 10,
        ..get_default_app_settings()
    }
}
