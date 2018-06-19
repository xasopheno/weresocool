pub struct Settings {
    pub sample_rate: f32,
    pub yin_buffer_size: f32,
    pub input_buffer_size: f32,
    pub output_buffer_size: f32,
    pub threshold: f32,
    pub gain_threshold: f32,
    pub channels: i32,
    pub interleaved: bool,
}

pub fn get_default_app_settings() -> &'static Settings {
    &Settings {
        sample_rate: 44_100.0,
        yin_buffer_size: 2048.0,
        input_buffer_size: 256.0,
        output_buffer_size: 256.0,
        threshold: 0.20,
        gain_threshold: 0.0,
        channels: 1,
        interleaved: true,
    }
}
